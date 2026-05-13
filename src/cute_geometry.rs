use crate::cute_settings::CuteSettings;
#[cfg(windows)]
use windows_sys::Win32::Graphics::Gdi::{EnumDisplayMonitors, HDC, HMONITOR};
#[cfg(windows)]
use windows_sys::Win32::Foundation::{RECT, LPARAM, BOOL};

#[derive(serde::Serialize, serde::Deserialize, Clone, Default)]
pub struct WindowGeometry {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[cfg(windows)]
unsafe extern "system" fn monitor_enum_proc(
    _hmonitor: HMONITOR,
    _hdc: HDC,
    lprect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let rects = &mut *(lparam as *mut Vec<RECT>);
    if !lprect.is_null() {
        rects.push(*lprect);
    }
    1 // TRUE
}

pub fn save_geometry(window: &slint::Window, settings: &mut CuteSettings) {
    let pos = window.position();
    let size = window.size();
    
    let geo = WindowGeometry {
        x: pos.x as f32,
        y: pos.y as f32,
        width: size.width as f32,
        height: size.height as f32,
    };
    
    settings.set_value("window_geometry", geo);
    settings.sync();
}

pub fn restore_geometry(window: &slint::Window, settings: &CuteSettings) {
    let geo = settings.value::<WindowGeometry>("window_geometry", WindowGeometry::default());
    
    if geo.width > 0.0 && geo.height > 0.0 {
        let mut is_visible = true;

        #[cfg(windows)]
        unsafe {
            let mut monitors: Vec<RECT> = Vec::new();
            EnumDisplayMonitors(
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                Some(monitor_enum_proc),
                &mut monitors as *mut _ as LPARAM,
            );

            // Verificar si el punto (x, y) cae dentro de algún monitor
            let x = geo.x as i32;
            let y = geo.y as i32;
            let mut intersects = false;
            for m in monitors {
                if x >= m.left && x <= m.right && y >= m.top && y <= m.bottom {
                    intersects = true;
                    break;
                }
            }
            if !intersects {
                is_visible = false;
            }
        }

        if is_visible {
            window.set_position(slint::PhysicalPosition::new(geo.x as i32, geo.y as i32));
            window.set_size(slint::PhysicalSize::new(geo.width as u32, geo.height as u32));
        }
    }
}
