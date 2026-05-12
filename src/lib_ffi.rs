use std::ffi::c_void;
use crate::ffi;

/// Initializes the SlintQT framework.
/// Returns 0 on success.
#[unsafe(no_mangle)]
pub extern "C" fn sq_init() -> i32 {
    // Perform any global initialization here (e.g. logging, environment variables).
    0
}

/// Creates the main application window and returns an opaque pointer.
/// This acts as the primary entry point for C++.
#[unsafe(no_mangle)]
pub extern "C" fn sq_create_app() -> *mut c_void {
    ffi::sq_window_new()
}
