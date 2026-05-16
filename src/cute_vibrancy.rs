use slint::Window;
use window_vibrancy::{apply_vibrancy, apply_mica, NSVisualEffectMaterial, NSVisualEffectState};

/// Applies native glassmorphism (Vibrancy on macOS, Mica/Acrylic on Windows) to the window.
pub fn apply_native_blur(window: &Window, is_dark: bool) {
    #[cfg(target_os = "windows")]
    {
        use raw_window_handle::HasWindowHandle;
        let handle = window.window_handle();
        let _ = apply_mica(&handle, Some(is_dark));
    }
    
    #[cfg(target_os = "macos")]
    {
        use raw_window_handle::HasWindowHandle;
        let material = if is_dark {
            NSVisualEffectMaterial::Dark
        } else {
            NSVisualEffectMaterial::Light
        };
        let handle = window.window_handle();
        let _ = apply_vibrancy(&handle, material, Some(NSVisualEffectState::Active), None);
    }
}
