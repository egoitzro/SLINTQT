use windows::core::{implement, Result};
use windows::Win32::System::Ole::{IDropTarget, IDropTarget_Impl, DROPEFFECT, DROPEFFECT_COPY, DROPEFFECT_NONE};
use windows::Win32::System::Com::{IDataObject, FORMATETC, STGMEDIUM, DVASPECT_CONTENT, TYMED_HGLOBAL};
use windows::Win32::Foundation::{POINTL, HWND};
use windows::Win32::UI::Input::KeyboardAndMouse::MODIFIERKEYS_FLAGS;
use windows::Win32::UI::Shell::{DragQueryFileW, HDROP};
use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref DROP_CALLBACK: Mutex<Option<Box<dyn Fn(Vec<String>) + Send + Sync>>> = Mutex::new(None);
}

#[implement(IDropTarget)]
pub struct NativeDropTarget {}

impl IDropTarget_Impl for NativeDropTarget {
    fn DragEnter(&self, _pDataObj: windows::core::r#ref::Ref<'_, IDataObject>, _grfKeyState: MODIFIERKEYS_FLAGS, _pt: &POINTL, pdwEffect: *mut DROPEFFECT) -> Result<()> {
        unsafe { *pdwEffect = DROPEFFECT_COPY; }
        Ok(())
    }
    
    fn DragOver(&self, _grfKeyState: MODIFIERKEYS_FLAGS, _pt: &POINTL, pdwEffect: *mut DROPEFFECT) -> Result<()> {
        unsafe { *pdwEffect = DROPEFFECT_COPY; }
        Ok(())
    }
    
    fn DragLeave(&self) -> Result<()> {
        Ok(())
    }
    
    fn Drop(&self, pDataObj: windows::core::r#ref::Ref<'_, IDataObject>, _grfKeyState: MODIFIERKEYS_FLAGS, _pt: &POINTL, pdwEffect: *mut DROPEFFECT) -> Result<()> {
        unsafe { *pdwEffect = DROPEFFECT_COPY; }
        
        let mut format = FORMATETC {
            cfFormat: 15, // CF_HDROP
            ptd: std::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT.0,
            lindex: -1,
            tymed: TYMED_HGLOBAL.0 as u32,
        };
        
        let mut medium = STGMEDIUM::default();
        if unsafe { pDataObj.GetData(&mut format, &mut medium) }.is_ok() {
            unsafe {
                let hglobal = medium.u.hGlobal;
                let ptr = GlobalLock(hglobal);
                if !ptr.is_null() {
                    let hdrop = HDROP(ptr as _);
                    let count = DragQueryFileW(hdrop, 0xFFFFFFFF, None);
                    let mut paths = Vec::new();
                    for i in 0..count {
                        let mut buf = [0u16; 260];
                        DragQueryFileW(hdrop, i, Some(&mut buf));
                        let len = buf.iter().take_while(|&&c| c != 0).count();
                        if let Ok(s) = String::from_utf16(&buf[..len]) {
                            paths.push(s);
                        }
                    }
                    if let Ok(cb) = DROP_CALLBACK.lock() {
                        if let Some(ref func) = *cb {
                            func(paths);
                        }
                    }
                    GlobalUnlock(hglobal);
                }
            }
        }
        
        Ok(())
    }
}
