#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
use fm_core::Result;
use std::{net::SocketAddr, path::PathBuf};
#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Folder-Mind: {e}");
    }
}
async fn run() -> Result<()> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let profile = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from))
        .unwrap_or_else(|| std::env::temp_dir().join("folder-mind-profile"));
    let pointer = std::env::var_os("FOLDER_MIND_POINTER")
        .map(PathBuf::from)
        .unwrap_or_else(|| profile.join("folder-mind/data-dir.txt"));
    let state = fm_web::AppState::new(pointer)?;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8484").await?;
    if std::env::var_os("FOLDER_MIND_NO_BROWSER").is_none() {
        open_browser();
    }
    tokio::spawn(fm_web::organization::background(state.clone()));
    tokio::spawn(fm_web::network::serve(state.clone()));
    let log_state = state.clone();
    tracing_subscriber::fmt()
        .with_target(false)
        .with_ansi(false)
        .with_writer(move || LogWriter {
            state: log_state.clone(),
            bytes: Vec::new(),
        })
        .init();
    let router = fm_web::router(state);
    if let Ok(ipv6) = tokio::net::TcpListener::bind("[::1]:8484").await {
        let r = router.clone();
        tokio::spawn(async move {
            let _ = axum_serve(ipv6, r).await;
        });
    }
    axum_serve(listener, router).await?;
    Ok(())
}
async fn axum_serve(
    listener: tokio::net::TcpListener,
    router: axum::Router,
) -> std::io::Result<()> {
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
}
fn open_browser() {
    #[cfg(windows)]
    let _ = std::process::Command::new("cmd")
        .args(["/C", "start", "", "http://127.0.0.1:8484"])
        .spawn();
    #[cfg(not(windows))]
    let _ = std::process::Command::new("xdg-open")
        .arg("http://127.0.0.1:8484")
        .spawn();
}

struct LogWriter {
    state: fm_web::AppState,
    bytes: Vec<u8>,
}
impl std::io::Write for LogWriter {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.bytes.extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl Drop for LogWriter {
    fn drop(&mut self) {
        if let Ok(l) = self.state.lib() {
            let _ = l.guard.append("protokolle/folder-mind.log", &self.bytes);
        }
    }
}
