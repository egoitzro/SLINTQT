#[cfg(windows)]
pub mod win32 {
    use std::ptr;
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::System::Com::{CoInitializeEx, CoCreateInstance, COINIT_APARTMENTTHREADED, CLSCTX_INPROC_SERVER};
    use windows_sys::Win32::UI::Shell::{
        SetWindowSubclass, DefSubclassProc,
        Shell_NotifyIconW, NOTIFYICONDATAW, TaskbarList, NIM_ADD, NIM_DELETE,
        NIF_ICON, NIF_MESSAGE, NIF_TIP, TBPF_NORMAL, TBPF_NOPROGRESS
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SendMessageW, WM_USER, WM_SETICON, ICON_SMALL, ICON_BIG,
        LoadImageW, IMAGE_ICON, LR_DEFAULTSIZE, LR_LOADFROMFILE,
        WM_LBUTTONUP, WM_RBUTTONUP, WM_DESTROY,
        FindWindowW, WM_COPYDATA, WM_HOTKEY, WM_SETTINGCHANGE,
    };
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::RegisterHotKey;
    use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
    use windows_sys::Win32::Graphics::Dwm::{DwmSetWindowAttribute};
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    /// Extrae el HWND de un handle genérico
    fn get_hwnd(window: &slint::Window) -> Option<HWND> {
        if let Ok(handle) = window.window_handle().window_handle() {
            if let raw_window_handle::RawWindowHandle::Win32(w) = handle.as_raw() {
                return Some(w.hwnd.get() as HWND);
            }
        }
        None
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        pub fn CreateMutexW(lpMutexAttributes: *const std::ffi::c_void, bInitialOwner: i32, lpName: *const u16) -> isize;
    }

    #[repr(C)]
    pub struct COPYDATASTRUCT {
        pub dwData: usize,
        pub cbData: u32,
        pub lpData: *mut std::ffi::c_void,
    }

    pub const WM_TRAY_ICON: u32 = WM_USER + 1;
    pub const DWMWA_USE_IMMERSIVE_DARK_MODE: u32 = 20;

    #[repr(C)]
    pub struct ITaskbarList3 {
        pub lpVtbl: *const ITaskbarList3Vtbl,
    }

    #[repr(C)]
    pub struct ITaskbarList3Vtbl {
        pub QueryInterface: unsafe extern "system" fn() -> i32,
        pub AddRef: unsafe extern "system" fn() -> u32,
        pub Release: unsafe extern "system" fn() -> u32,
        pub HrInit: unsafe extern "system" fn(*mut ITaskbarList3) -> i32,
        pub AddTab: unsafe extern "system" fn() -> i32,
        pub DeleteTab: unsafe extern "system" fn() -> i32,
        pub ActivateTab: unsafe extern "system" fn() -> i32,
        pub SetActiveAlt: unsafe extern "system" fn() -> i32,
        pub MarkFullscreenWindow: unsafe extern "system" fn() -> i32,
        pub SetProgressValue: unsafe extern "system" fn(*mut ITaskbarList3, HWND, u64, u64) -> i32,
        pub SetProgressState: unsafe extern "system" fn(*mut ITaskbarList3, HWND, u32) -> i32,
        pub RegisterTab: unsafe extern "system" fn() -> i32,
        pub UnregisterTab: unsafe extern "system" fn() -> i32,
        pub SetTabOrder: unsafe extern "system" fn() -> i32,
        pub SetTabActive: unsafe extern "system" fn() -> i32,
        pub ThumbBarAddButtons: unsafe extern "system" fn() -> i32,
        pub ThumbBarUpdateButtons: unsafe extern "system" fn() -> i32,
        pub ThumbBarSetImageList: unsafe extern "system" fn() -> i32,
        pub SetOverlayIcon: unsafe extern "system" fn() -> i32,
        pub SetThumbnailTooltip: unsafe extern "system" fn() -> i32,
        pub SetThumbnailClip: unsafe extern "system" fn() -> i32,
    }

    pub const IID_ITASKBARLIST3: windows_sys::core::GUID = windows_sys::core::GUID {
        data1: 0xea1afb91,
        data2: 0x9e28,
        data3: 0x4b86,
        data4: [0x90, 0xe9, 0x9e, 0x9f, 0x8a, 0x5e, 0xef, 0xaf],
    };

    /// Procedimiento de subclase para interceptar mensajes nativos
    unsafe extern "system" fn subclass_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        _id_subclass: usize,
        _ref_data: usize,
    ) -> LRESULT {
        unsafe {
            match msg {
                WM_COPYDATA => {
                    let cds = lparam as *const COPYDATASTRUCT;
                    if !cds.is_null() {
                        let ptr = (*cds).lpData as *const u8;
                        let len = (*cds).cbData as usize;
                        let slice = std::slice::from_raw_parts(ptr, len);
                        if let Ok(s) = std::str::from_utf8(slice) {
                            println!("[Win32] IPC Message Received: {}", s);
                        }
                    }
                    return 1;
                }
                WM_HOTKEY => {
                    let id = wparam as i32;
                    println!("[Win32] Global Hotkey pressed: {}", id);
                }
                WM_SETTINGCHANGE => {
                    let param = lparam as *const u16;
                    if !param.is_null() {
                        let mut len = 0;
                        while *param.offset(len) != 0 { len += 1; }
                        let slice = std::slice::from_raw_parts(param, len as usize);
                        if let Ok(s) = String::from_utf16(slice) {
                            if s == "ImmersiveColorSet" {
                                println!("[Win32] Theme changed dynamically!");
                            }
                        }
                    }
                }
                WM_TRAY_ICON => {
                    let event = lparam as u32;
                    match event {
                        WM_LBUTTONUP => {
                            println!("[Win32] Tray icon clic izquierdo");
                            // Restaurar ventana
                        }
                        WM_RBUTTONUP => {
                            println!("[Win32] Tray icon clic derecho");
                            // Mostrar menú
                        }
                        _ => {}
                    }
                }
                WM_DESTROY => {
                    let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
                    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
                    nid.hWnd = hwnd;
                    nid.uID = 1;
                    Shell_NotifyIconW(NIM_DELETE, &nid);
                }
                _ => {}
            }
            DefSubclassProc(hwnd, msg, wparam, lparam)
        }
    }

    /// 2. Taskbar Progress
    pub struct TaskbarManager {
        taskbar: *mut ITaskbarList3,
    }

    impl TaskbarManager {
        pub fn new() -> Result<Self, &'static str> {
            unsafe {
                CoInitializeEx(ptr::null_mut(), COINIT_APARTMENTTHREADED as u32);
                
                let mut taskbar: *mut std::ffi::c_void = ptr::null_mut();
                let hr = CoCreateInstance(
                    &TaskbarList,
                    ptr::null_mut(),
                    CLSCTX_INPROC_SERVER,
                    &IID_ITASKBARLIST3,
                    &mut taskbar,
                );
                
                if hr == 0 && !taskbar.is_null() {
                    let tb = taskbar as *mut ITaskbarList3;
                    if let Some(vtbl) = (*tb).lpVtbl.as_ref() {
                        (vtbl.HrInit)(tb);
                    }
                    Ok(Self { taskbar: tb })
                } else {
                    Err("Failed to create ITaskbarList3")
                }
            }
        }

        pub fn set_progress(&self, window: &slint::Window, completed: u64, total: u64) {
            if let Some(hwnd) = get_hwnd(window) {
                unsafe {
                    if let Some(vtbl) = (*self.taskbar).lpVtbl.as_ref() {
                        (vtbl.SetProgressValue)(self.taskbar, hwnd, completed, total);
                        (vtbl.SetProgressState)(self.taskbar, hwnd, TBPF_NORMAL as u32);
                    }
                }
            }
        }

        pub fn clear_progress(&self, window: &slint::Window) {
            if let Some(hwnd) = get_hwnd(window) {
                unsafe {
                    if let Some(vtbl) = (*self.taskbar).lpVtbl.as_ref() {
                        (vtbl.SetProgressState)(self.taskbar, hwnd, TBPF_NOPROGRESS as u32);
                    }
                }
            }
        }
    }

    /// 3. System Tray
    pub fn setup_system_tray(window: &slint::Window, tooltip: &str) -> Result<(), &'static str> {
        let hwnd = get_hwnd(window).ok_or("Invalid HWND")?;
        
        unsafe {
            // Aseguramos que el subclass esté instalado
            SetWindowSubclass(hwnd, Some(subclass_proc), 1, 0);

            let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd;
            nid.uID = 1;
            nid.uFlags = NIF_MESSAGE | NIF_TIP | NIF_ICON;
            nid.uCallbackMessage = WM_TRAY_ICON;
            
            // Cargar el ícono de la aplicación actual si existe, si no LoadImageA 
            // Para simplicidad, podemos usar el ícono predeterminado de la aplicación.
            // Para cargar desde el ejecutable (id 1):
            // nid.hIcon = LoadIconW(GetModuleHandleW(ptr::null()), MAKEINTRESOURCEW(1));
            
            let tooltip_wide: Vec<u16> = tooltip.encode_utf16().chain(std::iter::once(0)).collect();
            let len = tooltip_wide.len().min(128);
            nid.szTip[..len].copy_from_slice(&tooltip_wide[..len]);

            Shell_NotifyIconW(NIM_ADD, &nid);
        }
        Ok(())
    }

    /// 4. Window Icon (inyectado)
    pub fn set_window_icon_from_file(window: &slint::Window, path: &str) -> Result<(), &'static str> {
        let hwnd = get_hwnd(window).ok_or("Invalid HWND")?;
        
        let path_wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
        
        unsafe {
            let hicon = LoadImageW(
                ptr::null_mut(),
                path_wide.as_ptr(),
                IMAGE_ICON,
                0, 0,
                LR_LOADFROMFILE | LR_DEFAULTSIZE
            );
            
            if hicon.is_null() {
                return Err("Failed to load icon from file");
            }

            SendMessageW(hwnd, WM_SETICON, ICON_SMALL as usize, hicon as isize);
            SendMessageW(hwnd, WM_SETICON, ICON_BIG as usize, hicon as isize);
        }
        Ok(())
    }
    
    /// 5. Instancia Única e IPC
    pub fn check_single_instance(app_id: &str, window_title: &str, payload: &str) -> bool {
        let app_id_wide: Vec<u16> = app_id.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe {
            let _h_mutex = CreateMutexW(ptr::null(), 0, app_id_wide.as_ptr());
            if GetLastError() == ERROR_ALREADY_EXISTS {
                let title_wide: Vec<u16> = window_title.encode_utf16().chain(std::iter::once(0)).collect();
                let hwnd = FindWindowW(ptr::null(), title_wide.as_ptr());
                if !hwnd.is_null() {
                    let mut payload_utf8 = payload.to_string();
                    let cds = COPYDATASTRUCT {
                        dwData: 0,
                        cbData: payload_utf8.len() as u32,
                        lpData: payload_utf8.as_mut_ptr() as *mut std::ffi::c_void,
                    };
                    SendMessageW(hwnd, WM_COPYDATA, 0, &cds as *const _ as isize);
                }
                return true;
            }
        }
        false
    }

    /// 6. Barra de título oscura
    pub fn set_dark_title_bar(window: &slint::Window, dark: bool) {
        if let Some(hwnd) = get_hwnd(window) {
            let enable: i32 = if dark { 1 } else { 0 };
            unsafe {
                DwmSetWindowAttribute(
                    hwnd,
                    DWMWA_USE_IMMERSIVE_DARK_MODE,
                    &enable as *const _ as *const std::ffi::c_void,
                    std::mem::size_of::<i32>() as u32,
                );
            }
        }
    }

    /// 7. Global Hotkeys
    pub fn register_global_hotkey(window: &slint::Window, id: i32, modifiers: u32, vk: u32) {
        if let Some(hwnd) = get_hwnd(window) {
            unsafe {
                RegisterHotKey(hwnd, id, modifiers, vk);
            }
        }
    }
}
