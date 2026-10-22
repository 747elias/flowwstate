mod agents;
mod ai;
mod backup;
pub mod inbox;
mod markdown;
pub mod network;
pub mod organization;
mod semantic;
mod update;
use axum::{
    Json, Router,
    extract::{ConnectInfo, Query, Request, State},
    http::{StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use fm_core::{Chunk, Config, Document, Guard, Result, error};
use rust_embed::Embed;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicBool, Ordering},
    },
};
#[derive(Embed)]
#[folder = "../../web/"]
struct Assets;
#[derive(Clone)]
pub struct AppState {
    pub library: Arc<RwLock<Option<Arc<Library>>>>,
    pub pointer: PathBuf,
    pub csrf: String,
    pub paused: Arc<AtomicBool>,
    pub sessions: network::Sessions,
    pub attempts: network::Attempts,
    pub network_error: Arc<RwLock<String>>,
    pub ai_jobs: Arc<tokio::sync::Semaphore>,
    pub shutdown: Arc<AtomicBool>,
    pub inbox_jobs: Arc<tokio::sync::Semaphore>,
    pub updating: Arc<AtomicBool>,
}
pub struct Library {
    pub guard: Guard,
    pub config: RwLock<Config>,
    pub catalog: fm_catalog::Catalog,
    pub index: fm_index::SearchIndex,
    pub processing: Mutex<()>,
    pub stability: Mutex<fm_watcher::Stability>,
}
impl Library {
    pub fn open(guard: Guard) -> Result<Self> {
        let config = Config::load(&guard)?;
        let catalog = fm_catalog::Catalog::open(&guard)?;
        guard.mkdir("input")?;
        guard.mkdir("eingangsarchive")?;
        let index = fm_index::SearchIndex::open(&guard)?;
        index.rebuild(&catalog.chunks()?)?;
        Ok(Self {
            guard,
            config: RwLock::new(config),
            catalog,
            index,
            processing: Mutex::new(()),
            stability: Mutex::new(fm_watcher::Stability::default()),
        })
    }
    pub fn scan(&self, force: bool) -> Result<()> {
        let _lock = self.processing.lock().map_err(error)?;
        let known = self.catalog.documents()?;
        let mut changed = false;
        let mut files = Vec::new();
        walk(&self.guard, "unterlagen", &mut files)?;
        let max = self.config.read().map_err(error)?.max_file_mb * 1024 * 1024;
        for relative in &files {
            let relative = relative.clone();
            let path = self.guard.resolve(&relative)?;
            let meta = std::fs::metadata(&path)?;
            if meta.len() > max {
                self.catalog.record(
                    &Document {
                        path: relative,
                        hash: String::new(),
                        size: meta.len(),
                        status: "fehlgeschlagen".into(),
                        error: "Datei überschreitet die eingestellte Maximalgrösse".into(),
                    },
                    &[],
                )?;
                continue;
            }
            let wait = self.config.read().map_err(error)?.stable_seconds;
            if !self.stability.lock().map_err(error)?.ready(
                &relative,
                meta.len(),
                meta.modified()?,
                wait,
            ) {
                if !known.iter().any(|d| d.path == relative) {
                    self.catalog.record(
                        &Document {
                            path: relative.clone(),
                            hash: String::new(),
                            size: meta.len(),
                            status: "wartet_auf_stabile_datei".into(),
                            error: String::new(),
                        },
                        &[],
                    )?;
                }
                continue;
            }
            let bytes = std::fs::read(&path)?;
            let after = std::fs::metadata(&path)?;
            if meta.len() != after.len() || meta.modified()? != after.modified()? {
                self.stability.lock().map_err(error)?.forget(&relative);
                continue;
            }
            let hash = fm_core::hash(&bytes);
            if !force && known.iter().any(|d| d.path == relative && d.hash == hash) {
                continue;
            }
            changed = true;
            let mut d = Document {
                path: relative.clone(),
                hash,
                size: bytes.len() as u64,
                status: "geprüft".into(),
                error: String::new(),
            };
            self.catalog.record(&d, &[])?;
            if !fm_extract::supported(&relative) {
                d.status = "abgelegt".into();
                self.catalog.record(&d, &[])?;
                continue;
            }
            match fm_extract::extract(&relative, &bytes) {
                Ok(chunks) => {
                    d.status = "extrahiert".into();
                    self.catalog.record(&d, &chunks)?;
                    self.guard.write(
                        format!("extrakte/{}.json", fm_core::hash(relative.as_bytes())),
                        &serde_json::to_vec_pretty(&chunks).map_err(error)?,
                    )?;
                    d.status = "indiziert".into();
                    self.catalog.record(&d, &chunks)?;
                }
                Err(e) => {
                    d.status = "fehlgeschlagen".into();
                    d.error = e.to_string();
                    self.catalog.record(&d, &[])?;
                }
            }
        }
        for old in known.iter().filter(|d| !files.contains(&d.path)) {
            changed = true;
            self.catalog
                .execute("DELETE FROM documents WHERE path=?1", &[&old.path])?;
            self.catalog
                .execute("DELETE FROM jobs WHERE path=?1", &[&old.path])?;
        }
        if changed {
            self.index.rebuild(&self.catalog.chunks()?)?;
        }
        Ok(())
    }
}
pub fn walk(guard: &Guard, relative: &str, out: &mut Vec<String>) -> Result<()> {
    for entry in std::fs::read_dir(guard.resolve(relative)?)? {
        let e = entry?;
        let rel = format!("{relative}/{}", e.file_name().to_string_lossy());
        let Ok(path) = guard.resolve(&rel) else {
            continue;
        };
        if path.is_dir() {
            walk(guard, &rel, out)?;
        } else if path.is_file() {
            out.push(rel);
        }
    }
    Ok(())
}
impl AppState {
    pub fn new(pointer: PathBuf) -> Result<Self> {
        let library = if pointer.exists() {
            let p = std::fs::read_to_string(&pointer)?;
            Some(Arc::new(Library::open(Guard::new(p.trim())?)?))
        } else {
            None
        };
        Ok(Self {
            library: Arc::new(RwLock::new(library)),
            pointer,
            csrf: uuid::Uuid::new_v4().to_string(),
            paused: Arc::new(AtomicBool::new(false)),
            sessions: Arc::new(Mutex::new(Default::default())),
            attempts: Arc::new(Mutex::new(Default::default())),
            network_error: Arc::new(RwLock::new(String::new())),
            ai_jobs: Arc::new(tokio::sync::Semaphore::new(1)),
            shutdown: Arc::new(AtomicBool::new(false)),
            inbox_jobs: Arc::new(tokio::sync::Semaphore::new(1)),
            updating: Arc::new(AtomicBool::new(false)),
        })
    }
    pub fn lib(&self) -> Result<Arc<Library>> {
        self.library
            .read()
            .map_err(error)?
            .clone()
            .ok_or_else(|| error("Einrichtung zuerst abschliessen"))
    }
}
type ApiResult<T> = std::result::Result<T, ApiError>;
#[derive(Debug)]
pub struct ApiError(fm_core::Error);
impl From<fm_core::Error> for ApiError {
    fn from(e: fm_core::Error) -> Self {
        Self(e)
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error":self.0.to_string()})),
        )
            .into_response()
    }
}
fn peer(request: &Request) -> IpAddr {
    request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|p| p.0.ip())
        .unwrap_or(IpAddr::from([0, 0, 0, 0]))
}
pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(ai::routes())
        .merge(network::routes())
        .merge(organization::routes())
        .merge(backup::routes())
        .merge(semantic::routes())
        .merge(agents::routes())
        .merge(inbox::routes())
        .merge(update::routes())
        .route("/api/state", get(state_info))
        .route("/api/setup", post(setup))
        .route("/api/browse", get(browse))
        .route("/api/browse/create", post(browse_create))
        .route("/api/documents", get(documents))
        .route("/api/scan", post(scan))
        .route("/api/search", get(search))
        .route("/api/source", get(source))
        .route("/api/dashboard", get(dashboard))
        .fallback(get(asset))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            network::security,
        ))
        .with_state(state)
}
async fn state_info(State(s): State<AppState>, request: Request) -> Json<Value> {
    Json(
        json!({"product":"folder-mind","configured":s.lib().is_ok(),"csrf":s.csrf,"version":env!("CARGO_PKG_VERSION"),"login_required":!network::authenticated(&s,&request,peer(&request)),"is_local":peer(&request).is_loopback(),"allow_free_chat":s.lib().ok().and_then(|l|l.config.read().ok().map(|c|c.allow_free_chat)).unwrap_or(true)}),
    )
}
#[derive(Deserialize)]
struct Setup {
    path: String,
    #[serde(default)]
    name: String,
    #[serde(default = "default_template")]
    template: String,
    config: Option<Config>,
    api_key: Option<String>,
    password: Option<String>,
}
fn default_template() -> String {
    "lehre-informatik".into()
}
async fn setup(State(s): State<AppState>, Json(p): Json<Setup>) -> ApiResult<Json<Value>> {
    if s.lib().is_ok() {
        return Err(error("Einrichtung bereits abgeschlossen").into());
    }
    let guard = Guard::new(&p.path)?;
    let mut config = Config {
        name: p.name,
        template: p.template,
        ..p.config.unwrap_or_default()
    };
    config.levels = match config.template.as_str() {
        "schule" => vec!["Fach".into(), "Thema".into()],
        "frei" => vec!["Bereich".into()],
        "lehre-allgemein" => vec!["Lehrjahr".into(), "Fach".into(), "Thema".into()],
        _ => config.levels,
    };
    if let Some(password) = p.password
        && !password.is_empty()
    {
        config.password_hash = fm_access::password_hash(&password)?;
    }
    if !config.network_enabled {
        config.password_hash.clear();
    }
    config.api_key_ref = format!(
        "folder-mind-{}",
        fm_core::hash(guard.root().to_string_lossy().as_bytes())
    );
    if let Some(key) = p.api_key
        && !key.is_empty()
    {
        fm_ai::store_key(&config.api_key_ref, &key)?;
    }
    fm_access::validate(&config)?;
    fm_core::initialize(&guard, &config)?;
    let lib = Arc::new(Library::open(guard)?);
    // Der einzige Zeiger ausserhalb des Datenordners gehört zum Benutzerprofil.
    if let Some(parent) = s.pointer.parent() {
        std::fs::create_dir_all(parent).map_err(fm_core::Error::from)?;
    }
    std::fs::write(&s.pointer, lib.guard.root().to_string_lossy().as_bytes())
        .map_err(fm_core::Error::from)?;
    *s.library.write().map_err(error)? = Some(lib);
    Ok(Json(json!({"ok":true})))
}
#[derive(Deserialize)]
struct BrowseQuery {
    #[serde(default)]
    path: String,
}
async fn browse(Query(q): Query<BrowseQuery>) -> ApiResult<Json<Value>> {
    if q.path.is_empty() {
        #[cfg(windows)]
        let roots: Vec<String> = (b'A'..=b'Z')
            .map(|c| format!("{}:/", c as char))
            .filter(|s| Path::new(s).exists())
            .collect();
        #[cfg(not(windows))]
        let roots = vec!["/workspace".to_string(), "/tmp".to_string()];
        return Ok(Json(json!({"path":"","parent":"","directories":roots})));
    }
    let path = dunce_path(&q.path)?;
    let mut dirs = Vec::new();
    for entry in std::fs::read_dir(&path)
        .map_err(fm_core::Error::from)?
        .flatten()
    {
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            dirs.push(entry.path().to_string_lossy().to_string());
        }
    }
    dirs.sort();
    Ok(Json(
        json!({"path":path,"parent":path.parent(),"directories":dirs}),
    ))
}
fn dunce_path(path: &str) -> Result<PathBuf> {
    fm_core::Guard::new(path).map(|g| g.root().to_path_buf())
}
#[derive(Deserialize)]
struct CreateFolder {
    path: String,
    name: String,
}
async fn browse_create(Json(p): Json<CreateFolder>) -> ApiResult<Json<Value>> {
    let parent = Guard::new(&p.path)?;
    if Path::new(&p.name).components().count() != 1 {
        return Err(error("Bitte einen einzelnen Ordnernamen eingeben").into());
    }
    parent.mkdir(&p.name)?;
    Ok(Json(json!({"ok":true})))
}
async fn documents(State(s): State<AppState>) -> ApiResult<Json<Value>> {
    let lib = s.lib()?;
    let mut dirs = Vec::new();
    fn tree(g: &Guard, p: &str, out: &mut Vec<String>) -> Result<()> {
        out.push(p.into());
        for e in std::fs::read_dir(g.resolve(p)?)? {
            let e = e?;
            let rel = format!("{p}/{}", e.file_name().to_string_lossy());
            if let Ok(p) = g.resolve(&rel)
                && p.is_dir()
            {
                tree(g, &rel, out)?;
            }
        }
        Ok(())
    }
    tree(&lib.guard, "unterlagen", &mut dirs)?;
    Ok(Json(
        json!({"documents":lib.catalog.documents()?,"directories":dirs}),
    ))
}
async fn scan(State(s): State<AppState>) -> ApiResult<Json<Value>> {
    if !s.paused.load(Ordering::Relaxed) {
        let lib = s.lib()?;
        tokio::task::spawn_blocking(move || lib.scan(false))
            .await
            .map_err(error)??;
    }
    Ok(Json(json!({"ok":true})))
}
#[derive(Deserialize)]
struct SearchQuery {
    #[serde(default)]
    q: String,
    #[serde(default)]
    scope: String,
}
async fn search(
    State(s): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> ApiResult<Json<Vec<Chunk>>> {
    let l = s.lib()?;
    let ids = l.index.search(&q.q, 100)?;
    Ok(Json(
        l.catalog
            .chunks()?
            .into_iter()
            .filter(|c| ids.contains(&c.id) && in_scope(&c.path, &q.scope))
            .collect(),
    ))
}
pub fn in_scope(path: &str, scope: &str) -> bool {
    scope.is_empty()
        || path == scope
        || path.starts_with(&format!("{}/", scope.trim_end_matches('/')))
}
#[derive(Deserialize)]
struct SourceQuery {
    id: String,
    #[serde(default)]
    download: bool,
}
async fn source(State(s): State<AppState>, Query(q): Query<SourceQuery>) -> ApiResult<Response> {
    if q.id.len() != 64 || !q.id.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(error("Ungültige Quellenkennung").into());
    }
    let l = s.lib()?;
    let c = l
        .catalog
        .source(&q.id)?
        .ok_or_else(|| error("Quelle nicht gefunden"))?;
    let path = l.guard.resolve(&c.path)?;
    let body = if path.exists() {
        Some(std::fs::read(path).map_err(fm_core::Error::from)?)
    } else {
        None
    };
    let current = body.as_ref().is_some_and(|b| fm_core::hash(b) == c.hash);
    if current && c.path.to_lowercase().ends_with(".pdf") {
        return Ok(([(header::CONTENT_TYPE, "application/pdf")], body.unwrap()).into_response());
    }
    if q.download {
        if !current {
            return Err(error("Das Original ist nicht mehr im damaligen Stand vorhanden. Der archivierte Extrakt bleibt sichtbar.").into());
        }
        let body = body.unwrap_or_default();
        let name = Path::new(&c.path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        let encoded = url::form_urlencoded::byte_serialize(name.as_bytes())
            .collect::<String>()
            .replace('+', "%20");
        return Ok((
            [
                (header::CONTENT_TYPE, "application/octet-stream".to_string()),
                (
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename*=UTF-8''{encoded}"),
                ),
            ],
            body,
        )
            .into_response());
    }
    let esc = |text: &str| {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('\"', "&quot;")
    };
    let html = format!(
        "<!doctype html><html lang=\"de\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Quelle · Folder-Mind</title><link rel=\"stylesheet\" href=\"/style.css\"></head><body><main><h1>{}</h1><p>{}</p><article class=\"card\"><pre>{}</pre></article><a href=\"/api/source?id={}&amp;download=true\" download>Original herunterladen</a><p>SHA-256: {}</p></main></body></html>",
        esc(&c.path),
        esc(&format!(
            "{}{}",
            c.location,
            if current {
                ""
            } else {
                " · Archivierter Extrakt: Original geändert oder entfernt"
            }
        )),
        esc(&c.text),
        c.id,
        esc(&c.hash)
    );
    Ok(axum::response::Html(html).into_response())
}

async fn dashboard(State(s): State<AppState>) -> ApiResult<Json<Value>> {
    let l = s.lib()?;
    let d = l.catalog.documents()?;
    Ok(Json(
        json!({"documents":d.len(),"errors":d.iter().filter(|d|d.status=="fehlgeschlagen").count(),"pending":l.catalog.rows("SELECT path FROM knowledge_pages WHERE status='wartet_auf_freigabe'")?.len(),"path":l.guard.root(),"paused":s.paused.load(Ordering::Relaxed),"recent_chats":l.catalog.rows("SELECT id,title FROM chats ORDER BY rowid DESC LIMIT 5")?,"recent_documents":l.catalog.rows("SELECT path,status FROM documents ORDER BY rowid DESC LIMIT 5")?}),
    ))
}
async fn asset(request: Request) -> Response {
    let path = request.uri().path().trim_start_matches('/');
    let file = if path.contains('.') {
        path
    } else {
        "index.html"
    };
    if let Some(a) = Assets::get(file) {
        let mime = match file.rsplit('.').next() {
            Some("js") => "text/javascript; charset=utf-8",
            Some("css") => "text/css; charset=utf-8",
            Some("txt") => "text/plain; charset=utf-8",
            Some("svg") => "image/svg+xml",
            _ => "text/html; charset=utf-8",
        };
        ([(header::CONTENT_TYPE, mime)], a.data.into_owned()).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}
