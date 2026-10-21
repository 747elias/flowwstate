#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
#[cfg(windows)]
mod tray;
use fm_core::Result;
use std::{net::SocketAddr, path::PathBuf};
#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        show_error(&e.to_string());
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
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:8484").await {
        Ok(l) => l,
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            let http = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(2))
                .build()
                .map_err(fm_core::error)?;
            if let Ok(response) = http.get("http://127.0.0.1:8484/api/state").send().await
                && let Ok(info) = response.json::<serde_json::Value>().await
                && info["product"] == "folder-mind"
            {
                open_browser();
                return Ok(());
            }
            return Err(fm_core::error(
                "Port 8484 ist bereits durch ein anderes Programm belegt.",
            ));
        }
        Err(e) => return Err(e.into()),
    };
    let state = fm_web::AppState::new(pointer)?;
    if std::env::var_os("FOLDER_MIND_NO_BROWSER").is_none() {
        open_browser();
    }
    tokio::spawn(fm_web::organization::background(state.clone()));
    tokio::spawn(fm_web::network::serve(state.clone()));
    let log_state = state.clone();
    let (log_tx, log_rx) = std::sync::mpsc::sync_channel::<Vec<u8>>(1024);
    std::thread::spawn(move || {
        while let Ok(bytes) = log_rx.recv() {
            if let Ok(l) = log_state.lib() {
                let _ = l.guard.append("protokolle/folder-mind.log", &bytes);
            }
        }
    });
    tracing_subscriber::fmt()
        .with_target(false)
        .with_ansi(false)
        .with_writer(move || LogWriter {
            sender: log_tx.clone(),
            bytes: Vec::new(),
        })
        .init();
    #[cfg(windows)]
    {
        let tray_state = state.clone();
        std::thread::spawn(move || {
            if let Err(e) = tray::run(tray_state.clone()) {
                show_error(&format!("Tray-Icon konnte nicht gestartet werden: {e}"));
                tray_state
                    .shutdown
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
        });
    }
    let shutdown = state.shutdown.clone();
    let router = fm_web::router(state);
    if let Ok(ipv6) = tokio::net::TcpListener::bind("[::1]:8484").await {
        let r = router.clone();
        let stop = shutdown.clone();
        tokio::spawn(async move {
            let _ = axum_serve(ipv6, r, stop).await;
        });
    }
    axum_serve(listener, router, shutdown).await?;
    Ok(())
}
async fn axum_serve(
    listener: tokio::net::TcpListener,
    router: axum::Router,
    shutdown: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> std::io::Result<()> {
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async move {
        while !shutdown.load(std::sync::atomic::Ordering::Relaxed) {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
    })
    .await
}
fn open_browser() {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", "http://127.0.0.1:8484"])
            .creation_flags(0x08000000)
            .spawn();
    }
    #[cfg(not(windows))]
    let _ = std::process::Command::new("xdg-open")
        .arg("http://127.0.0.1:8484")
        .spawn();
}

struct LogWriter {
    sender: std::sync::mpsc::SyncSender<Vec<u8>>,
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
        let _ = self.sender.try_send(std::mem::take(&mut self.bytes));
    }
}

fn show_error(message: &str) {
    #[cfg(windows)]
    unsafe {
        use windows::{
            Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW},
            core::{PCWSTR, w},
        };
        let text = message
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        MessageBoxW(
            None,
            PCWSTR(text.as_ptr()),
            w!("Folder-Mind"),
            MB_OK | MB_ICONERROR,
        );
    }
    #[cfg(not(windows))]
    eprintln!("Folder-Mind: {message}");
}
