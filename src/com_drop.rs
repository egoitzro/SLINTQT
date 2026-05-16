#![allow(non_snake_case)]
use windows_sys::Win32::System::Com::{FORMATETC, STGMEDIUM, DVASPECT_CONTENT, TYMED_HGLOBAL};
use windows_sys::Win32::System::Ole::{RegisterDragDrop, RevokeDragDrop, DROPEFFECT_COPY, CF_HDROP};
use windows_sys::Win32::Foundation::{HWND, POINTL, S_OK};
use windows_sys::Win32::UI::Shell::DragQueryFileW;
use windows_sys::Win32::System::Memory::GlobalLock;
use windows_sys::Win32::UI::WindowsAndMessaging::{ChangeWindowMessageFilterEx, MSGFLT_ALLOW, WM_DROPFILES, WM_COPYDATA};
use windows_sys::core::GUID;
use std::ffi::c_void;
use std::ptr;

#[repr(C)]
pub struct IUnknownVtbl {
    pub QueryInterface: unsafe extern "system" fn(this: *mut c_void, riid: *const GUID, ppvObject: *mut *mut c_void) -> i32,
    pub AddRef: unsafe extern "system" fn(this: *mut c_void) -> u32,
    pub Release: unsafe extern "system" fn(this: *mut c_void) -> u32,
}

#[repr(C)]
pub struct IDataObjectVtbl {
    pub base: IUnknownVtbl,
    pub GetData: unsafe extern "system" fn(this: *mut c_void, pformatetcIn: *const FORMATETC, pmedium: *mut STGMEDIUM) -> i32,
    // other methods omitted as we only need GetData (at index 3)
}

#[repr(C)]
pub struct IDataObject {
    pub vtbl: *const IDataObjectVtbl,
}

#[repr(C)]
pub struct IDropTargetVtbl {
    pub base: IUnknownVtbl,
    pub DragEnter: unsafe extern "system" fn(this: *mut c_void, pDataObj: *mut c_void, grfKeyState: u32, pt: POINTL, pdwEffect: *mut u32) -> i32,
    pub DragOver: unsafe extern "system" fn(this: *mut c_void, grfKeyState: u32, pt: POINTL, pdwEffect: *mut u32) -> i32,
    pub DragLeave: unsafe extern "system" fn(this: *mut c_void) -> i32,
    pub Drop: unsafe extern "system" fn(this: *mut c_void, pDataObj: *mut c_void, grfKeyState: u32, pt: POINTL, pdwEffect: *mut u32) -> i32,
}

#[repr(C)]
pub struct MyDropTarget {
    pub vtbl: *const IDropTargetVtbl,
    pub ref_count: u32,
    pub callback: Option<Box<dyn Fn(Vec<String>) + Send + Sync>>,
}

unsafe extern "system" fn QueryInterface(this: *mut c_void, _riid: *const GUID, ppvObject: *mut *mut c_void) -> i32 {
    // We only support IUnknown and IDropTarget
    *ppvObject = this;
    S_OK
}

unsafe extern "system" fn AddRef(this: *mut c_void) -> u32 {
    let target = this as *mut MyDropTarget;
    (*target).ref_count += 1;
    (*target).ref_count
}

unsafe extern "system" fn Release(this: *mut c_void) -> u32 {
    let target = this as *mut MyDropTarget;
    (*target).ref_count -= 1;
    let count = (*target).ref_count;
    if count == 0 {
        let _ = Box::from_raw(target); // Drop the memory and callback
    }
    count
}

unsafe extern "system" fn DragEnter(_this: *mut c_void, _pDataObj: *mut c_void, _grfKeyState: u32, _pt: POINTL, pdwEffect: *mut u32) -> i32 {
    *pdwEffect = DROPEFFECT_COPY;
    S_OK
}

unsafe extern "system" fn DragOver(_this: *mut c_void, _grfKeyState: u32, _pt: POINTL, pdwEffect: *mut u32) -> i32 {
    *pdwEffect = DROPEFFECT_COPY;
    S_OK
}

unsafe extern "system" fn DragLeave(_this: *mut c_void) -> i32 {
    S_OK
}

unsafe extern "system" fn Drop(_this: *mut c_void, pDataObj: *mut c_void, _grfKeyState: u32, _pt: POINTL, pdwEffect: *mut u32) -> i32 {
    *pdwEffect = DROPEFFECT_COPY;
    
    let data_obj = pDataObj as *mut IDataObject;
    let format = FORMATETC {
        cfFormat: CF_HDROP as u16,
        ptd: ptr::null_mut(),
        dwAspect: DVASPECT_CONTENT as u32,
        lindex: -1,
        tymed: TYMED_HGLOBAL as u32,
    };
    
    let mut medium = STGMEDIUM { tymed: 0, u: std::mem::zeroed(), pUnkForRelease: ptr::null_mut() };
    
    let hr = ((*(*data_obj).vtbl).GetData)(pDataObj, &format, &mut medium);
    if hr == S_OK {
        let hglobal = medium.u.hGlobal;
        let p_drop = GlobalLock(hglobal);
        if !p_drop.is_null() {
            let hdrop = p_drop;
            let count = DragQueryFileW(hdrop, 0xFFFFFFFF, ptr::null_mut(), 0);
            let mut paths = Vec::new();
            for i in 0..count {
                let mut buf = [0u16; 260];
                DragQueryFileW(hdrop, i, buf.as_mut_ptr(), 260);
                let len = buf.iter().take_while(|&&c| c != 0).count();
                if let Ok(s) = String::from_utf16(&buf[..len]) {
                    paths.push(s);
                }
            }
            let target = _this as *mut MyDropTarget;
            if let Some(ref func) = (*target).callback {
                func(paths);
            }
        }
    }
    
    S_OK
}

static DROP_TARGET_VTBL: IDropTargetVtbl = IDropTargetVtbl {
    base: IUnknownVtbl {
        QueryInterface,
        AddRef,
        Release,
    },
    DragEnter,
    DragOver,
    DragLeave,
    Drop,
};

pub fn setup_com_drop(window: &slint::Window, callback: impl Fn(Vec<String>) + Send + Sync + 'static) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    
    let hwnd = match window.window_handle().window_handle() {
        Ok(handle) => match handle.as_raw() {
            RawWindowHandle::Win32(w) => w.hwnd.get() as isize as HWND,
            _ => { println!("[COM Drop] Not a Win32 window"); return; },
        },
        Err(e) => { println!("[COM Drop] Error getting window handle: {:?}", e); return; },
    };

    unsafe {
        RevokeDragDrop(hwnd); // Revoke winit's
        
        let target = Box::new(MyDropTarget {
            vtbl: &DROP_TARGET_VTBL,
            ref_count: 1,
            callback: Some(Box::new(callback)),
        });
        
        // Leak the box so it stays alive for COM.
        let target_ptr = Box::into_raw(target) as *mut c_void;
        // UIPI Bypass
        ChangeWindowMessageFilterEx(hwnd, WM_DROPFILES, MSGFLT_ALLOW, ptr::null_mut());
        ChangeWindowMessageFilterEx(hwnd, WM_COPYDATA, MSGFLT_ALLOW, ptr::null_mut());
        ChangeWindowMessageFilterEx(hwnd, 0x0049, MSGFLT_ALLOW, ptr::null_mut()); // WM_COPYGLOBALDATA

        let res = RegisterDragDrop(hwnd, target_ptr);
        println!("[COM Drop] RegisterDragDrop returned: {:X}", res);
    }
}
