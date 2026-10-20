//! Window material: Windows 11 Mica behind the transparent main window.

/// Applies Mica to the window, tinted dark or light (`None` follows the
/// system). Returns whether it took: false before Windows 11 and on other
/// systems, where the page paints its own background instead.
#[tauri::command]
pub fn apply_window_material(window: tauri::WebviewWindow, dark: Option<bool>) -> bool {
    #[cfg(windows)]
    {
        window_vibrancy::apply_mica(&window, dark).is_ok()
    }
    #[cfg(not(windows))]
    {
        let _ = (window, dark);
        false
    }
}
