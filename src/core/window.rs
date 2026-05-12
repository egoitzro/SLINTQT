use slint::{ComponentHandle, Global};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use crate::CuteMainWindow;

pub fn initialize_window(ui: &CuteMainWindow) {
    // Platform Bridge Logic: Detect OS and set theme
    use crate::ThemeEngine;
    
    #[cfg(target_os = "windows")]
    {
        ui.set_platform_theme(crate::PlatformTheme::Windows);
        ThemeEngine::get(ui).set_current_theme(crate::PlatformTheme::Windows);
        
        let handle = ui.window().window_handle();
        if let Ok(raw_handle) = handle.window_handle()
            && let RawWindowHandle::Win32(win_handle) = raw_handle.as_raw() {
                let hwnd = win_handle.hwnd.get() as *mut std::ffi::c_void;
                
                // 1. Efecto MICA
                let backdrop_type: i32 = 2; // DWMSBT_MAINWINDOW (Mica)
                unsafe {
                    windows_sys::Win32::Graphics::Dwm::DwmSetWindowAttribute(
                        hwnd,
                        38, // DWMWA_SYSTEMBACKDROP_TYPE
                        &backdrop_type as *const i32 as *const std::ffi::c_void,
                        std::mem::size_of::<i32>() as u32,
                    );
                }

                // 2. Bordes redondeados nativos de Windows 11
                let corner_preference: i32 = 2; // DWMWCP_ROUND
                unsafe {
                    windows_sys::Win32::Graphics::Dwm::DwmSetWindowAttribute(
                        hwnd,
                        33, // DWMWA_WINDOW_CORNER_PREFERENCE
                        &corner_preference as *const i32 as *const std::ffi::c_void,
                        std::mem::size_of::<i32>() as u32,
                    );
                }

                // 3. Forzar Modo Claro
                let dark_mode: i32 = 0; 
                unsafe {
                    windows_sys::Win32::Graphics::Dwm::DwmSetWindowAttribute(
                        hwnd,
                        20, // DWMWA_USE_IMMERSIVE_DARK_MODE
                        &dark_mode as *const i32 as *const std::ffi::c_void,
                        std::mem::size_of::<i32>() as u32,
                    );
                }
                println!("Efecto Mica y Bordes Redondeados Nativos habilitados vía DWM");
            }
    }
    
    #[cfg(target_os = "macos")]
    {
        ui.set_platform_theme(crate::PlatformTheme::MacOS);
        ThemeEngine::get(ui).set_current_theme(crate::PlatformTheme::MacOS);
    }
    
    #[cfg(target_os = "linux")]
    {
        ui.set_platform_theme(crate::PlatformTheme::LinuxKde);
        ThemeEngine::get(ui).set_current_theme(crate::PlatformTheme::LinuxKde);
    }

    // Conexiones de manejo de ventana (Win32)
    setup_window_controls(ui);
}

fn setup_window_controls(ui: &CuteMainWindow) {
    #[cfg(target_os = "windows")]
    {
        use crate::cute_connect;
        
        let ui_handle_drag = ui.as_weak();
        cute_connect!(ui, on_start_drag, move || {
            if let Some(ui) = ui_handle_drag.upgrade() {
                let handle = ui.window().window_handle();
                if let Ok(raw_handle) = handle.window_handle()
                    && let RawWindowHandle::Win32(win_handle) = raw_handle.as_raw() {
                        let hwnd = win_handle.hwnd.get() as *mut std::ffi::c_void;
                        unsafe {
                            windows_sys::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture();
                            windows_sys::Win32::UI::WindowsAndMessaging::SendMessageW(
                                hwnd,
                                windows_sys::Win32::UI::WindowsAndMessaging::WM_NCLBUTTONDOWN,
                                windows_sys::Win32::UI::WindowsAndMessaging::HTCAPTION as usize,
                                0,
                            );
                            windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW(
                                hwnd,
                                windows_sys::Win32::UI::WindowsAndMessaging::WM_LBUTTONUP,
                                0,
                                0,
                            );
                        }
                    }
            }
        });

        let ui_handle_min = ui.as_weak();
        cute_connect!(ui, on_minimize_application, move || {
            if let Some(ui) = ui_handle_min.upgrade() {
                let handle = ui.window().window_handle();
                if let Ok(raw_handle) = handle.window_handle()
                    && let RawWindowHandle::Win32(win_handle) = raw_handle.as_raw() {
                        let hwnd = win_handle.hwnd.get() as *mut std::ffi::c_void;
                        unsafe {
                            windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW(
                                hwnd,
                                windows_sys::Win32::UI::WindowsAndMessaging::WM_SYSCOMMAND,
                                windows_sys::Win32::UI::WindowsAndMessaging::SC_MINIMIZE as usize,
                                0,
                            );
                        }
                    }
            }
        });

        let ui_handle_max = ui.as_weak();
        cute_connect!(ui, on_maximize_application, move || {
            if let Some(ui) = ui_handle_max.upgrade() {
                let handle = ui.window().window_handle();
                if let Ok(raw_handle) = handle.window_handle()
                    && let RawWindowHandle::Win32(win_handle) = raw_handle.as_raw() {
                        let hwnd = win_handle.hwnd.get() as *mut std::ffi::c_void;
                        unsafe {
                            let mut wp: windows_sys::Win32::UI::WindowsAndMessaging::WINDOWPLACEMENT = std::mem::zeroed();
                            wp.length = std::mem::size_of::<windows_sys::Win32::UI::WindowsAndMessaging::WINDOWPLACEMENT>() as u32;
                            windows_sys::Win32::UI::WindowsAndMessaging::GetWindowPlacement(hwnd, &mut wp);

                            if wp.showCmd == windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWMAXIMIZED as u32 {
                                windows_sys::Win32::UI::WindowsAndMessaging::ShowWindow(
                                    hwnd,
                                    windows_sys::Win32::UI::WindowsAndMessaging::SW_RESTORE,
                                );
                                ui.set_is_maximized(false);
                            } else {
                                windows_sys::Win32::UI::WindowsAndMessaging::ShowWindow(
                                    hwnd,
                                    windows_sys::Win32::UI::WindowsAndMessaging::SW_MAXIMIZE,
                                );
                                ui.set_is_maximized(true);
                            }
                        }
                    }
            }
        });

        let ui_handle_close = ui.as_weak();
        cute_connect!(ui, on_close_application, move || {
            if let Some(ui) = ui_handle_close.upgrade() {
                let handle = ui.window().window_handle();
                if let Ok(raw_handle) = handle.window_handle()
                    && let RawWindowHandle::Win32(win_handle) = raw_handle.as_raw() {
                        let hwnd = win_handle.hwnd.get() as *mut std::ffi::c_void;
                        unsafe {
                            windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW(
                                hwnd,
                                windows_sys::Win32::UI::WindowsAndMessaging::WM_SYSCOMMAND,
                                windows_sys::Win32::UI::WindowsAndMessaging::SC_CLOSE as usize,
                                0,
                            );
                        }
                        let _ = slint::quit_event_loop();
                    }
            }
        });
    }
}
