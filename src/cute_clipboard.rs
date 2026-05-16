#[cfg(windows)]
use windows_sys::Win32::System::DataExchange::{OpenClipboard, CloseClipboard, GetClipboardData, SetClipboardData, EmptyClipboard};
#[cfg(windows)]
use windows_sys::Win32::System::Memory::{GlobalLock, GlobalUnlock, GlobalAlloc, GHND};
#[cfg(windows)]
use windows_sys::Win32::UI::Shell::{DragQueryFileW, DROPFILES};
#[cfg(windows)]
use windows_sys::Win32::Foundation::HWND;

pub struct CuteClipboard;

impl CuteClipboard {
    pub fn get_files() -> Vec<String> {
        let mut paths = Vec::new();
        #[cfg(windows)]
        unsafe {
            if OpenClipboard(0 as HWND) != 0 {
                let hdrop = GetClipboardData(15); // CF_HDROP = 15
                if hdrop != std::ptr::null_mut() {
                    let hdrop = hdrop as *mut std::ffi::c_void;
                    let count = DragQueryFileW(hdrop, 0xFFFFFFFF, std::ptr::null_mut(), 0);
                    for i in 0..count {
                        let mut buffer = [0u16; 260];
                        DragQueryFileW(hdrop, i, buffer.as_mut_ptr(), 260);
                        let len = buffer.iter().take_while(|&&c| c != 0).count();
                        if let Ok(s) = String::from_utf16(&buffer[..len]) {
                            paths.push(s);
                        }
                    }
                }
                CloseClipboard();
            }
        }
        paths
    }

    pub fn set_files(paths: &[String]) -> Result<(), &'static str> {
        #[cfg(windows)]
        unsafe {
            if OpenClipboard(0 as HWND) == 0 {
                return Err("Cannot open clipboard");
            }
            EmptyClipboard();

            let mut buffer: Vec<u16> = Vec::new();
            for path in paths {
                for c in path.encode_utf16() {
                    buffer.push(c);
                }
                buffer.push(0);
            }
            buffer.push(0); // Double null termination

            let size = std::mem::size_of::<DROPFILES>() + buffer.len() * 2;
            let hglobal = GlobalAlloc(GHND, size);
            if hglobal == std::ptr::null_mut() {
                CloseClipboard();
                return Err("Failed to allocate memory");
            }

            let ptr = GlobalLock(hglobal) as *mut u8;
            let dropfiles = ptr as *mut DROPFILES;
            (*dropfiles).pFiles = std::mem::size_of::<DROPFILES>() as u32;
            (*dropfiles).fWide = 1; // TRUE

            let data_ptr = ptr.add(std::mem::size_of::<DROPFILES>());
            std::ptr::copy_nonoverlapping(buffer.as_ptr() as *const u8, data_ptr, buffer.len() * 2);

            GlobalUnlock(hglobal);
            SetClipboardData(15, hglobal); // CF_HDROP = 15
            CloseClipboard();
            Ok(())
        }
        #[cfg(not(windows))]
        Err("Not implemented for non-windows")
    }
}
