pub mod core;
pub mod cute_signals;
pub mod ffi;
pub mod lib_ffi;
pub mod docking;

// Expose all generated Slint components
slint::include_modules!();
