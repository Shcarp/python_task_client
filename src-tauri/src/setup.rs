use tauri::{App, Manager};
#[cfg(target_os = "macos")]
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};
#[cfg(target_os = "windows")]
use window_vibrancy::apply_blur;

/// setup
pub fn init(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let win = app.get_window("main").unwrap();

    #[cfg(target_os = "macos")]
    apply_vibrancy(&win, NSVisualEffectMaterial::HudWindow, None, None).expect("Unsupported platform! 'apply_vibrancy' is only supported on macOS");

    #[cfg(target_os = "windows")]
    apply_blur(&window, Some((18, 18, 18, 125))).expect("Unsupported platform! 'apply_blur' is only supported on Windows");
    
    Ok(())
}