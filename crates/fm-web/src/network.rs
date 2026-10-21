use super::*;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
pub type Sessions = Arc<Mutex<HashMap<String, (IpAddr, Instant)>>>;
pub type Attempts = Arc<Mutex<HashMap<IpAddr, (u8, Instant)>>>;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/login", post(login))
        .route("/api/logout", post(logout))
        .route("/api/access/status", get(status))
        .route("/api/access/resolve", get(resolve))
        .route("/api/access/allow", post(allow_device))
}
pub fn authenticated(s: &AppState, request: &Request, ip: IpAddr) -> bool {
    if ip.is_loopback() {
        return true;
    }
    let cookie = request
        .headers()
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let token = cookie
        .split(';')
        .map(str::trim)
        .find_map(|c| c.strip_prefix("fm_session="));
    let Ok(mut sessions) = s.sessions.lock() else {
        return false;
    };
    sessions.retain(|_, (_, until)| *until > Instant::now());
    token
        .and_then(|t| sessions.get(t))
        .is_some_and(|(bound, _)| *bound == ip)
}
pub async fn security(State(s): State<AppState>, request: Request, next: Next) -> Response {
    let ip = fm_access::normalize(peer(&request));
    let path = request.uri().path();
    let local = ip.is_loopback();
    let admin = fm_access::is_admin(ip);
    if !local {
        let Ok(l) = s.lib() else {
            return (StatusCode::FORBIDDEN, "Zugriff verweigert").into_response();
        };
        let Ok(c) = l.config.read().map(|c| c.clone()) else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };
        let mac = if !fm_access::allowed(ip, &c, None) && !c.allowed_macs.is_empty() {
            match fm_access::resolve_mac(ip) {
                Ok(m) => Some(m),
                Err(e) => {
                    tracing::warn!("MAC-Auflösung: {e}");
                    if let Ok(mut m) = s.network_error.write() {
                        *m = e.to_string();
                    }
                    None
                }
            }
        } else {
            None
        };
        let allowed = fm_access::allowed(ip, &c, mac);
        let _ = l.catalog.execute(
            "INSERT INTO access_log(ip,allowed,created_at) VALUES(?1,?2,?3)",
            &[&ip.to_string(), &(allowed as i64), &fm_core::now()],
        );
        if !allowed {
            tracing::warn!("Zugriff von {ip} verweigert");
            return (
                StatusCode::FORBIDDEN,
                "Zugriff verweigert: Dieses Gerät ist nicht in der Whitelist.",
            )
                .into_response();
        }
    }
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let literal_host = host.parse::<SocketAddr>().is_ok();
    if !(literal_host || host == "localhost:8484") {
        return (StatusCode::FORBIDDEN, "Ungültiger Host").into_response();
    }
    if local
        && request.uri().scheme_str() != Some("https")
        && !host.ends_with(":8485")
        && !matches!(host, "127.0.0.1:8484" | "localhost:8484" | "[::1]:8484")
    {
        return (StatusCode::FORBIDDEN, "Ungültiger lokaler Host").into_response();
    }
    let administrative = path.starts_with("/api/settings")
        || path.starts_with("/api/access/")
        || path.starts_with("/api/browse")
        || path.starts_with("/api/setup")
        || path.starts_with("/api/ai/test")
        || path.starts_with("/api/backup")
        || path.starts_with("/api/data/");
    if administrative && !admin {
        return(StatusCode::FORBIDDEN,"Einstellungen und Freigaben sind nur über http://127.0.0.1:8484 auf diesem Computer änderbar.").into_response();
    }
    if !matches!(request.method().as_str(), "GET" | "HEAD" | "OPTIONS") {
        if request
            .headers()
            .get("x-csrf-token")
            .and_then(|v| v.to_str().ok())
            != Some(s.csrf.as_str())
        {
            return (StatusCode::FORBIDDEN, "CSRF-Prüfung fehlgeschlagen").into_response();
        }
        if let Some(origin) = request
            .headers()
            .get(header::ORIGIN)
            .and_then(|v| v.to_str().ok())
        {
            let scheme = if host.ends_with(":8484") {
                "http"
            } else {
                "https"
            };
            if origin != format!("{scheme}://{host}") {
                return (StatusCode::FORBIDDEN, "Ursprung nicht erlaubt").into_response();
            }
        }
    }
    let public = [
        "/api/state",
        "/api/login",
        "/anmeldung",
        "/app.js",
        "/style.css",
        "/htmx.min.js",
        "/favicon.svg",
        "/third-party.txt",
    ]
    .contains(&path);
    if !public && !authenticated(&s, &request, ip) {
        if path.starts_with("/api/") {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error":"Bitte zuerst anmelden"})),
            )
                .into_response();
        }
        return Redirect::to("/anmeldung").into_response();
    }
    if s.lib().is_err()
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
            "/favicon.svg",
            "/third-party.txt",
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
#[derive(Deserialize)]
struct Login {
    password: String,
}
async fn login(
    State(s): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(p): Json<Login>,
) -> ApiResult<Response> {
    let ip = fm_access::normalize(addr.ip());
    let l = s.lib()?;
    {
        let mut attempts = s.attempts.lock().map_err(error)?;
        attempts.retain(|_, (_, since)| since.elapsed() < Duration::from_secs(60));
        let entry = attempts.entry(ip).or_insert((0, Instant::now()));
        if entry.0 >= 5 {
            return Err(error("Zu viele Anmeldeversuche. Bitte eine Minute warten.").into());
        }
        entry.0 += 1;
    }
    let hash = l.config.read().map_err(error)?.password_hash.clone();
    let valid = tokio::task::spawn_blocking(move || fm_access::verify(&p.password, &hash))
        .await
        .map_err(error)?;
    if !valid {
        return Err(error("Passwort ist falsch").into());
    }
    s.attempts.lock().map_err(error)?.remove(&ip);
    let token = format!("{}{}", uuid::Uuid::new_v4(), uuid::Uuid::new_v4());
    s.sessions.lock().map_err(error)?.insert(
        token.clone(),
        (ip, Instant::now() + Duration::from_secs(8 * 3600)),
    );
    Ok((
        [(
            header::SET_COOKIE,
            format!("fm_session={token}; Secure; HttpOnly; SameSite=Strict; Path=/; Max-Age=28800"),
        )],
        Json(json!({"ok":true})),
    )
        .into_response())
}
async fn logout(State(s): State<AppState>, request: Request) -> Response {
    let ip = peer(&request);
    if let Ok(mut sessions) = s.sessions.lock() {
        sessions.retain(|_, (bound, _)| *bound != ip);
    }
    (
        [(
            header::SET_COOKIE,
            "fm_session=; Secure; HttpOnly; SameSite=Strict; Path=/; Max-Age=0",
        )],
        Json(json!({"ok":true})),
    )
        .into_response()
}
async fn status(State(s): State<AppState>) -> ApiResult<Json<Value>> {
    let l = s.lib()?;
    let c = l.config.read().map_err(error)?.clone();
    let thumbprint = if l.guard.resolve(".folder-mind/tls-cert.pem")?.exists() {
        fm_access::certificate_fingerprint(&l.guard)?
    } else {
        String::new()
    };
    Ok(Json(
        json!({"network_enabled":c.network_enabled,"bind":c.network_bind,"error":s.network_error.read().map_err(error)?.clone(),"certificate_sha256":thumbprint,"denied":l.catalog.rows("SELECT ip,CAST(created_at AS TEXT) FROM access_log WHERE allowed=0 ORDER BY id DESC LIMIT 30")?}),
    ))
}
#[derive(Deserialize)]
struct IpQuery {
    ip: String,
}
async fn resolve(Query(q): Query<IpQuery>) -> ApiResult<Json<Value>> {
    let ip = q.ip.parse().map_err(|_| error("Ungültige IP-Adresse"))?;
    let mac = fm_access::resolve_mac(ip)?;
    Ok(Json(
        json!({"mac":mac.iter().map(|b|format!("{b:02X}")).collect::<Vec<_>>().join(":" )}),
    ))
}
#[derive(Deserialize)]
struct Allow {
    ip: String,
    confirmed: bool,
}
async fn allow_device(State(s): State<AppState>, Json(p): Json<Allow>) -> ApiResult<Json<Value>> {
    if !p.confirmed {
        return Err(error("Bitte Freigabe bestätigen").into());
    }
    fm_access::parse_ip(&p.ip)?;
    let l = s.lib()?;
    let mut c = l.config.write().map_err(error)?;
    if !c.allowed_ips.contains(&p.ip) {
        c.allowed_ips.push(p.ip);
    }
    c.save(&l.guard)?;
    Ok(Json(json!({"ok":true})))
}
pub async fn serve(s: AppState) {
    let mut active: Option<(String, axum_server::Handle)> = None;
    loop {
        if s.shutdown.load(Ordering::Relaxed) {
            if let Some((_, handle)) = active.take() {
                handle.graceful_shutdown(Some(Duration::from_secs(2)));
            }
            break;
        }
        let wanted = s
            .lib()
            .ok()
            .and_then(|l| l.config.read().ok().map(|c| c.clone()))
            .filter(|c| c.network_enabled && fm_access::validate(c).is_ok());
        let binding = wanted.as_ref().map(|c| c.network_bind.clone());
        if active.as_ref().map(|v| &v.0) != binding.as_ref() {
            if let Some((_, handle)) = active.take() {
                handle.graceful_shutdown(Some(Duration::from_secs(2)));
            }
            if let Some(c) = wanted {
                let result=async{let l=s.lib()?;let(cert,key)=fm_access::tls_material(&l.guard,&c.network_bind)?;let tls=axum_server::tls_rustls::RustlsConfig::from_pem(cert,key).await.map_err(error)?;let addr:SocketAddr=c.network_bind.parse().map_err(error)?;let handle=axum_server::Handle::new();let h=handle.clone();let router=super::router(s.clone());let errors=s.network_error.clone();tokio::spawn(async move{if let Err(e)=axum_server::bind_rustls(addr,tls).handle(h).serve(router.into_make_service_with_connect_info::<SocketAddr>()).await{tracing::error!("HTTPS-Server konnte nicht gestartet werden: {e}");if let Ok(mut error)=errors.write(){*error="HTTPS-Listener konnte nicht gestartet werden. Schnittstelle und Port prüfen.".into();}}});Ok::<_,fm_core::Error>((c.network_bind,handle))}.await;
                match result {
                    Ok(v) => {
                        active = Some(v);
                    }
                    Err(e) => {
                        if let Ok(mut msg) = s.network_error.write() {
                            *msg = e.to_string();
                        }
                    }
                }
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
