use super::*;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/update", get(status))
        .route("/api/update/prepare", post(prepare))
}
async fn status(State(s): State<AppState>) -> ApiResult<Json<Value>> {
    let l = s.lib()?;
    let mut migrations = Vec::new();
    for entry in std::fs::read_dir(l.guard.resolve("backups")?).map_err(fm_core::Error::from)? {
        let entry = entry.map_err(fm_core::Error::from)?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("vor-update-") {
            migrations.push(name);
        }
    }
    migrations.sort();
    Ok(Json(
        json!({"version":env!("CARGO_PKG_VERSION"),"data_path":l.guard.root(),"migration_backups":migrations,"release_url":"https://github.com/747elias/folder-mind/releases/latest","updating":s.updating.load(Ordering::Relaxed)}),
    ))
}
#[derive(Deserialize)]
struct Prepare {
    confirmed: bool,
}
async fn prepare(State(s): State<AppState>, Json(p): Json<Prepare>) -> ApiResult<Json<Value>> {
    if !p.confirmed {
        return Err(error("Bitte bestätigen, dass die neue EXE heruntergeladen wurde und das Programm für das Update beendet werden darf.").into());
    }
    if s.updating.swap(true, Ordering::SeqCst) {
        return Err(error("Ein Update wird bereits vorbereitet").into());
    }
    let was_paused = s.paused.swap(true, Ordering::SeqCst);
    let result = async {
        let _inbox = s.inbox_jobs.clone().acquire_owned().await.map_err(error)?;
        let _ai = s.ai_jobs.clone().acquire_owned().await.map_err(error)?;
        let l = s.lib()?;
        tokio::task::spawn_blocking(move || super::backup::make(&l))
            .await
            .map_err(error)?
    }
    .await;
    match result {
        Ok(name) => {
            let stop = s.shutdown.clone();
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                stop.store(true, Ordering::Relaxed);
            });
            Ok(Json(
                json!({"ok":true,"backup":name,"message":"Sicherung erstellt. Folder-Mind wird beendet. Ersetze die bisherige EXE durch die heruntergeladene neue EXE und starte sie. Datenordner, Einstellungen, gespeicherte Chats, Agenten und Lernregeln bleiben erhalten."}),
            ))
        }
        Err(e) => {
            s.updating.store(false, Ordering::SeqCst);
            s.paused.store(was_paused, Ordering::SeqCst);
            Err(e.into())
        }
    }
}
