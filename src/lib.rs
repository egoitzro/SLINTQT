pub mod core;
pub mod cute_signals;
pub mod ffi;
pub mod lib_ffi;
pub mod docking;

// Expose all generated Slint components
slint::include_modules!();

pub mod win32_native;

pub mod cute_settings;
pub mod cute_geometry;
pub mod com_drop;
pub mod cute_msgbox;
pub mod cute_clipboard;
pub mod cute_i18n;
pub mod cute_tray;
pub mod cute_proxy;
pub mod cute_vibrancy;
