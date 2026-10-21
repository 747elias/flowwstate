mod ai;
use axum::{
    Json, Router,
    body::Body,
    extract::{ConnectInfo, Query, Request, State},
    http::{HeaderMap, StatusCode, header},
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
}
pub struct Library {
    pub guard: Guard,
    pub config: RwLock<Config>,
    pub catalog: fm_catalog::Catalog,
    pub index: fm_index::SearchIndex,
    pub processing: Mutex<()>,
}
impl Library {
    pub fn open(guard: Guard) -> Result<Self> {
        let config = Config::load(&guard)?;
        let catalog = fm_catalog::Catalog::open(&guard)?;
        let index = fm_index::SearchIndex::open(&guard)?;
        index.rebuild(&catalog.chunks()?)?;
        Ok(Self {
            guard,
            config: RwLock::new(config),
            catalog,
            index,
            processing: Mutex::new(()),
        })
    }
    pub fn scan(&self, force: bool) -> Result<()> {
        let _lock = self.processing.lock().map_err(error)?;
        let known = self.catalog.documents()?;
        let mut files = Vec::new();
        walk(&self.guard, "unterlagen", &mut files)?;
        let max = self.config.read().map_err(error)?.max_file_mb * 1024 * 1024;
        for relative in files {
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
            let bytes = std::fs::read(&path)?;
            let hash = fm_core::hash(&bytes);
            if !force && known.iter().any(|d| d.path == relative && d.hash == hash) {
                continue;
            }
            let mut d = Document {
                path: relative.clone(),
                hash,
                size: bytes.len() as u64,
                status: "gefunden".into(),
                error: String::new(),
            };
            self.catalog.record(&d, &[])?;
            match fm_extract::extract(&relative, &bytes) {
                Ok(chunks) => {
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
        self.index.rebuild(&self.catalog.chunks()?)?;
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
        .unwrap_or(IpAddr::from([127, 0, 0, 1]))
}
async fn security(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let ip = peer(&request);
    let path = request.uri().path();
    if !ip.is_loopback() {
        return (StatusCode::FORBIDDEN, "Zugriff verweigert").into_response();
    }
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    if !["127.0.0.1:8484", "localhost:8484", "[::1]:8484"].contains(&host) {
        return (StatusCode::FORBIDDEN, "Ungültiger Host").into_response();
    }
    if !matches!(request.method().as_str(), "GET" | "HEAD" | "OPTIONS") {
        if request
            .headers()
            .get("x-csrf-token")
            .and_then(|v| v.to_str().ok())
            != Some(state.csrf.as_str())
        {
            return (StatusCode::FORBIDDEN, "CSRF-Prüfung fehlgeschlagen").into_response();
        }
        if let Some(origin) = request
            .headers()
            .get(header::ORIGIN)
            .and_then(|v| v.to_str().ok())
            && origin != format!("http://{host}")
        {
            return (StatusCode::FORBIDDEN, "Ursprung nicht erlaubt").into_response();
        }
    }
    if state.lib().is_err()
        && ![
            "/api/state",
            "/api/setup",
            "/api/ai/test",
            "/api/browse",
            "/api/browse/create",
            "/einrichtung",
            "/app.js",
            "/style.css",
            "/htmx.min.js",
        ]
        .contains(&path)
    {
        return Redirect::to("/einrichtung").into_response();
    }
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("x-content-type-options", "nosniff".parse().unwrap());
    response.headers_mut().insert("content-security-policy","default-src 'self'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self' data:; frame-ancestors 'none'; base-uri 'none'; form-action 'self'".parse().unwrap());
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    response
}
pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(ai::routes())
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
        .layer(middleware::from_fn_with_state(state.clone(), security))
        .with_state(state)
}
async fn state_info(State(s): State<AppState>) -> Json<Value> {
    Json(json!({"configured":s.lib().is_ok(),"csrf":s.csrf,"version":env!("CARGO_PKG_VERSION")}))
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
    config.api_key_ref = format!(
        "folder-mind-{}",
        fm_core::hash(guard.root().to_string_lossy().as_bytes())
    );
    if let Some(key) = p.api_key
        && !key.is_empty()
    {
        fm_ai::store_key(&config.api_key_ref, &key)?;
    }
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
}
async fn source(State(s): State<AppState>, Query(q): Query<SourceQuery>) -> ApiResult<Response> {
    let l = s.lib()?;
    let c = l
        .catalog
        .chunks()?
        .into_iter()
        .find(|c| c.id == q.id)
        .ok_or_else(|| error("Quelle nicht gefunden"))?;
    let body = std::fs::read(l.guard.resolve(&c.path)?).map_err(fm_core::Error::from)?;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        if c.path.to_lowercase().ends_with(".pdf") {
            "application/pdf"
        } else {
            "text/plain; charset=utf-8"
        }
        .parse()
        .unwrap(),
    );
    Ok((headers, Body::from(body)).into_response())
}
async fn dashboard(State(s): State<AppState>) -> ApiResult<Json<Value>> {
    let l = s.lib()?;
    let d = l.catalog.documents()?;
    Ok(Json(
        json!({"documents":d.len(),"errors":d.iter().filter(|d|d.status=="fehlgeschlagen").count(),"pending":0,"path":l.guard.root(),"paused":s.paused.load(Ordering::Relaxed)}),
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
            _ => "text/html; charset=utf-8",
        };
        ([(header::CONTENT_TYPE, mime)], a.data.into_owned()).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}
