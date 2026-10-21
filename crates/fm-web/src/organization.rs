use super::*;
use axum::extract::{DefaultBodyLimit, Multipart};
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/folder", post(folder))
        .route("/api/upload", post(upload))
        .route("/api/queue", get(queue))
        .route("/api/queue/action", post(queue_action))
        .route("/api/protocol", get(protocol))
        .route("/api/health-fragment", get(health))
        .layer(DefaultBodyLimit::max(510 * 1024 * 1024))
}
fn original_path(l: &Library, path: &str) -> Result<PathBuf> {
    if path != "unterlagen" && !path.starts_with("unterlagen/") {
        return Err(error(
            "Diese Aktion ist nur innerhalb von unterlagen erlaubt",
        ));
    }
    l.guard.resolve(path)
}
fn folder_name(c: &Config, name: &str) -> Result<String> {
    if Path::new(name).components().count() != 1 || name.contains(['/', '\\']) {
        return Err(error("Bitte einen einzelnen Ordnernamen eingeben"));
    }
    let name = if c.naming_rules {
        name.trim().to_lowercase().replace(char::is_whitespace, "-")
    } else {
        name.into()
    };
    if name.is_empty() {
        return Err(error("Ordnername fehlt"));
    }
    Ok(name)
}
#[derive(Deserialize)]
struct FolderAction {
    action: String,
    path: String,
    name: Option<String>,
    target: Option<String>,
    #[serde(default)]
    confirmed: bool,
}
async fn folder(State(s): State<AppState>, Json(p): Json<FolderAction>) -> ApiResult<Json<Value>> {
    let l = s.lib()?;
    let _busy = l.processing.lock().map_err(error)?;
    original_path(&l, &p.path)?;
    let c = l.config.read().map_err(error)?.clone();
    match p.action.as_str() {
        "create" => {
            let name = folder_name(&c, &p.name.unwrap_or_default())?;
            l.guard.mkdir(format!("{}/{name}", p.path))?;
        }
        "rename" => {
            if p.path == "unterlagen" {
                return Err(error("Die Unterlagenwurzel kann nicht umbenannt werden").into());
            }
            let name = folder_name(&c, &p.name.unwrap_or_default())?;
            let parent = Path::new(&p.path)
                .parent()
                .ok_or_else(|| error("Ungültiger Pfad"))?;
            l.guard.move_path(&p.path, parent.join(name))?;
        }
        "move" => {
            let target = p.target.ok_or_else(|| error("Ziel fehlt"))?;
            original_path(&l, &target)?;
            if p.path == "unterlagen" {
                return Err(error("Die Unterlagenwurzel ist geschützt").into());
            }
            l.guard.move_path(&p.path, target)?;
        }
        "delete" => {
            if p.path == "unterlagen" {
                return Err(error("Die Unterlagenwurzel ist geschützt").into());
            }
            l.guard.trash(&p.path, p.confirmed)?;
        }
        _ => return Err(error("Unbekannte Ordneraktion").into()),
    }
    Ok(Json(json!({"ok":true})))
}
#[derive(Deserialize)]
struct UploadQuery {
    path: String,
}
async fn upload(
    State(s): State<AppState>,
    Query(q): Query<UploadQuery>,
    mut multipart: Multipart,
) -> ApiResult<Json<Value>> {
    let l = s.lib()?;
    let dir = original_path(&l, &q.path)?;
    if !dir.is_dir() {
        return Err(error("Zielordner fehlt").into());
    }
    let max = l.config.read().map_err(error)?.max_file_mb * 1024 * 1024;
    let mut names = Vec::new();
    while let Some(field) = multipart.next_field().await.map_err(error)? {
        let name = field
            .file_name()
            .ok_or_else(|| error("Dateiname fehlt"))?
            .to_string();
        if Path::new(&name).components().count() != 1 || name.contains(['/', '\\']) {
            return Err(error("Ungültiger Upload-Dateiname").into());
        }
        let bytes = field.bytes().await.map_err(error)?;
        if bytes.len() as u64 > max {
            return Err(error("Datei überschreitet die maximale Dateigrösse").into());
        }
        l.guard.upload(format!("{}/{name}", q.path), &bytes)?;
        names.push(name);
    }
    Ok(Json(json!({"ok":true,"files":names})))
}
async fn queue(State(s): State<AppState>) -> ApiResult<Json<Value>> {
    let l = s.lib()?;
    Ok(Json(
        json!({"documents":l.catalog.documents()?,"paused":s.paused.load(Ordering::Relaxed)}),
    ))
}
#[derive(Deserialize)]
struct QueueAction {
    action: String,
}
async fn queue_action(
    State(s): State<AppState>,
    Json(p): Json<QueueAction>,
) -> ApiResult<Json<Value>> {
    match p.action.as_str() {
        "pause" => s.paused.store(true, Ordering::Relaxed),
        "resume" => s.paused.store(false, Ordering::Relaxed),
        "retry" => {
            let l = s.lib()?;
            tokio::task::spawn_blocking(move || l.scan(true))
                .await
                .map_err(error)??;
        }
        "reindex" => {
            let l = s.lib()?;
            l.index.rebuild(&l.catalog.chunks()?)?;
        }
        _ => return Err(error("Unbekannte Warteschlangenaktion").into()),
    }
    Ok(Json(json!({"ok":true})))
}
async fn protocol(State(s): State<AppState>) -> ApiResult<Json<Value>> {
    let l = s.lib()?;
    let p = l.guard.resolve("protokolle/folder-mind.log")?;
    let log = if p.exists() {
        let bytes = std::fs::read(p).map_err(fm_core::Error::from)?;
        String::from_utf8_lossy(&bytes[bytes.len().saturating_sub(1_000_000)..]).to_string()
    } else {
        String::new()
    };
    Ok(Json(
        json!({"log":log,"access":l.catalog.rows("SELECT ip,CAST(allowed AS TEXT),CAST(created_at AS TEXT) FROM access_log ORDER BY id DESC LIMIT 100")?,"documents":l.catalog.documents()?}),
    ))
}
async fn health(State(s): State<AppState>) -> axum::response::Html<String> {
    let count = s
        .lib()
        .and_then(|l| l.catalog.documents())
        .map(|v| v.len())
        .unwrap_or(0);
    axum::response::Html(format!(
        "{count} Dateien · Verarbeitung {}",
        if s.paused.load(Ordering::Relaxed) {
            "pausiert"
        } else {
            "aktiv"
        }
    ))
}
pub async fn background(s: AppState) {
    let mut observed = PathBuf::new();
    let mut watcher = None;
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    loop {
        if let Ok(l) = s.lib() {
            if observed != l.guard.root() {
                observed = l.guard.root().to_path_buf();
                watcher = fm_watcher::watch(
                    &l.guard.resolve("unterlagen").unwrap_or_default(),
                    tx.clone(),
                )
                .ok();
            }
            if !s.paused.load(Ordering::Relaxed) {
                let lib = l.clone();
                if let Err(e) = tokio::task::spawn_blocking(move || lib.scan(false))
                    .await
                    .unwrap_or_else(|e| Err(error(e)))
                {
                    tracing::warn!("Verarbeitung fehlgeschlagen: {e}");
                }
            }
        }
        let _ = &watcher;
        tokio::select! {_=tokio::time::sleep(std::time::Duration::from_secs(5))=>{},_=rx.recv()=>{while rx.try_recv().is_ok(){}}}
    }
}
