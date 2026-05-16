use std::sync::{Arc, Mutex};
use tray_icon::{TrayIcon, TrayIconBuilder, Icon};
use muda::{Menu, MenuItem, PredefinedMenuItem};

pub struct CuteSystemTray {
    tray: Option<TrayIcon>,
    menu: Option<Menu>,
}

impl CuteSystemTray {
    pub fn new() -> Self {
        // Create an empty, simple tray icon for now.
        // It requires an icon. We will generate a blank 1x1 transparent icon just to not panic.
        let icon = match Self::create_blank_icon() {
            Ok(i) => Some(i),
            Err(_) => None,
        };

        let tray_menu = Menu::new();
        let show_i = MenuItem::new("Mostrar", true, None);
        let quit_i = MenuItem::new("Salir", true, None);
        let _ = tray_menu.append(&show_i);
        let _ = tray_menu.append(&PredefinedMenuItem::separator());
        let _ = tray_menu.append(&quit_i);

        let mut builder = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu.clone()))
            .with_tooltip("MKVToolNix Rust (SlintQT)");

        if let Some(i) = icon {
            builder = builder.with_icon(i);
        }

        let tray = match builder.build() {
            Ok(t) => Some(t),
            Err(e) => {
                eprintln!("[CuteSystemTray] Error building tray icon: {:?}", e);
                None
            }
        };

        Self { tray, menu: Some(tray_menu) }
    }

    pub fn show_message(&self, _title: &str, _msg: &str) {
        // tray-icon no soporta globos directamente en todas las plataformas.
        // Para notificaciones reales cruzadas se recomienda usar la crate `notify-rust`.
        // Mantenemos la firma para compatibilidad.
        println!("[CuteSystemTray] Notificación cruzada simulada: {} - {}", _title, _msg);
    }

    fn create_blank_icon() -> Result<Icon, tray_icon::BadIcon> {
        // 1x1 transparent RGBA pixel
        let rgba = vec![0, 0, 0, 0];
        Icon::from_rgba(rgba, 1, 1)
    }
}
