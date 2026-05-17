#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, MB_OK, MB_YESNO, MB_ICONINFORMATION, MB_ICONWARNING, MB_ICONERROR, MB_ICONQUESTION,
    IDOK, IDYES, IDNO
};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

pub enum CuteMessageBoxStyle {
    Information,
    Warning,
    Error,
    Question,
}

pub enum CuteMessageBoxResult {
    Ok,
    Yes,
    No,
    Cancel,
}

pub fn show_message_box(title: &str, text: &str, style: CuteMessageBoxStyle) -> CuteMessageBoxResult {
    #[cfg(windows)]
    unsafe {
        let flags = match style {
            CuteMessageBoxStyle::Information => MB_OK | MB_ICONINFORMATION,
            CuteMessageBoxStyle::Warning => MB_OK | MB_ICONWARNING,
            CuteMessageBoxStyle::Error => MB_OK | MB_ICONERROR,
            CuteMessageBoxStyle::Question => MB_YESNO | MB_ICONQUESTION,
        };

        let title_wide: Vec<u16> = OsStr::new(title).encode_wide().chain(std::iter::once(0)).collect();
        let text_wide: Vec<u16> = OsStr::new(text).encode_wide().chain(std::iter::once(0)).collect();

        let result = MessageBoxW(std::ptr::null_mut(), text_wide.as_ptr(), title_wide.as_ptr(), flags);

        match result {
            IDOK => CuteMessageBoxResult::Ok,
            IDYES => CuteMessageBoxResult::Yes,
            IDNO => CuteMessageBoxResult::No,
            _ => CuteMessageBoxResult::Cancel,
        }
    }
    
    #[cfg(not(windows))]
    {
        println!("[{}] {}", title, text);
        CuteMessageBoxResult::Ok
    }
}
