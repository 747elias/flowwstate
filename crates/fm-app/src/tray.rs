use fm_core::{Result, error};
use std::sync::atomic::Ordering;
pub fn run(state: fm_web::AppState) -> Result<()> {
    use tray_icon::{
        Icon, TrayIconBuilder,
        menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage,
    };
    let open = MenuItem::new("Öffnen", true, None);
    let pause = MenuItem::new("Pause", true, None);
    let quit = MenuItem::new("Beenden", true, None);
    let menu = Menu::new();
    menu.append(&open).map_err(error)?;
    menu.append(&pause).map_err(error)?;
    menu.append(&PredefinedMenuItem::separator())
        .map_err(error)?;
    menu.append(&quit).map_err(error)?;
    let mut rgba = vec![0u8; 32 * 32 * 4];
    for y in 6..27 {
        for x in 3..29 {
            if y >= 10 || x < 16 {
                let i = (y * 32 + x) * 4;
                rgba[i..i + 4].copy_from_slice(&[22, 130, 117, 255]);
            }
        }
    }
    let icon = Icon::from_rgba(rgba, 32, 32).map_err(error)?;
    let _tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Folder-Mind")
        .with_icon(icon)
        .build()
        .map_err(error)?;
    while !state.shutdown.load(Ordering::Relaxed) {
        unsafe {
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == open.id() {
                super::open_browser();
            } else if event.id == pause.id() {
                let paused = !state.paused.load(Ordering::Relaxed);
                state.paused.store(paused, Ordering::Relaxed);
                pause.set_text(if paused { "Fortsetzen" } else { "Pause" });
            } else if event.id == quit.id() {
                state.shutdown.store(true, Ordering::Relaxed);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    Ok(())
}
