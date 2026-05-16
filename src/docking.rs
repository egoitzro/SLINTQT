use crate::{CuteMainWindow, CuteFloatingWindow};
use slint::{ComponentHandle, Weak};
use std::rc::Rc;
use std::cell::RefCell;
use std::time::Instant;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LayoutState {
    pub dock_left_width: f32,
    pub splitter_fraction: f32,
    pub project_explorer_visible: bool,
    pub properties_visible: bool,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            dock_left_width: 250.0,
            splitter_fraction: 0.5,
            project_explorer_visible: true,
            properties_visible: true,
        }
    }
}

pub struct DockManager {
    main_window: Weak<CuteMainWindow>,
    layout_tx: Sender<LayoutState>,
}

impl DockManager {
    pub fn new(main_window: &CuteMainWindow) -> Self {
        let (tx, rx) = mpsc::channel::<LayoutState>();
        
        thread::spawn(move || {
            let mut current_state = None;
            loop {
                match rx.recv_timeout(Duration::from_millis(500)) {
                    Ok(state) => {
                        current_state = Some(state);
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        if let Some(state) = current_state.take() {
                            if let Ok(json) = serde_json::to_string_pretty(&state) {
                                let _ = fs::write("layout.json", json);
                            }
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        break;
                    }
                }
            }
        });
        
        let initial_state = if let Ok(json) = fs::read_to_string("layout.json") {
            serde_json::from_str(&json).unwrap_or_default()
        } else {
            LayoutState::default()
        };
        
        main_window.set_dock_left_width(initial_state.dock_left_width);
        main_window.set_splitter_fraction(initial_state.splitter_fraction);
        main_window.set_dock_project_explorer_visible(initial_state.project_explorer_visible);
        main_window.set_dock_properties_visible(initial_state.properties_visible);

        let manager = Self {
            main_window: main_window.as_weak(),
            layout_tx: tx,
        };
        manager.init_docking();
        manager.init_splitters();
        
        // Restore floating windows if they were detached
        if !initial_state.project_explorer_visible {
            main_window.invoke_dock_detached("Project Explorer".into(), 0.0, 0.0);
        }
        if !initial_state.properties_visible {
            main_window.invoke_dock_detached("Properties".into(), 0.0, 0.0);
        }
        
        manager
    }

    fn save_state(ui: &CuteMainWindow, tx: &Sender<LayoutState>) {
        let state = LayoutState {
            dock_left_width: ui.get_dock_left_width(),
            splitter_fraction: ui.get_splitter_fraction(),
            project_explorer_visible: ui.get_dock_project_explorer_visible(),
            properties_visible: ui.get_dock_properties_visible(),
        };
        let _ = tx.send(state);
    }

    fn init_docking(&self) {
        let ui_weak_dock = self.main_window.clone();
        let tx_detach = self.layout_tx.clone();
        
        if let Some(ui) = ui_weak_dock.upgrade() {
            ui.on_dock_detached(move |title, _x, _y| {
                let ui_handle = ui_weak_dock.upgrade().unwrap();
                
                // Spawn real floating native window
                let float_win = CuteFloatingWindow::new().unwrap();
                float_win.set_dock_title(title.clone());
                float_win.set_tree_data(ui_handle.get_tree_data());
                
                let title_clone = title.clone();
                if title_clone.as_str() == "Project Explorer" {
                    ui_handle.set_dock_project_explorer_visible(false);
                    float_win.set_content_type(0);
                } else if title_clone.as_str() == "Properties" {
                    ui_handle.set_dock_properties_visible(false);
                    float_win.set_content_type(1);
                }
                Self::save_state(&ui_handle, &tx_detach);
                
                let float_win_weak = float_win.as_weak();
                let ui_weak_close = ui_weak_dock.clone();
                let title_clone_close = title.clone();
                let tx_close = tx_detach.clone();
                
                float_win.on_close_window(move || {
                    if let Some(ui) = ui_weak_close.upgrade() {
                        if title_clone_close.as_str() == "Project Explorer" {
                            ui.set_dock_project_explorer_visible(true);
                        } else if title_clone_close.as_str() == "Properties" {
                            ui.set_dock_properties_visible(true);
                        }
                        Self::save_state(&ui, &tx_close);
                    }
                    if let Some(win) = float_win_weak.upgrade() {
                        win.hide().unwrap();
                    }
                });
                
                let ui_weak_tree1 = ui_weak_dock.clone();
                float_win.on_tree_item_clicked(move |idx, item| {
                    if let Some(ui) = ui_weak_tree1.upgrade() {
                        ui.invoke_tree_item_clicked(idx, item);
                    }
                });
                
                let ui_weak_tree2 = ui_weak_dock.clone();
                float_win.on_tree_toggle_expanded(move |idx, item| {
                    if let Some(ui) = ui_weak_tree2.upgrade() {
                        ui.invoke_tree_toggle_expanded(idx, item);
                    }
                });
                
                let initial_pos = Rc::new(RefCell::new(None));
                let last_update = Rc::new(RefCell::new(Instant::now()));

                let ip_clone = initial_pos.clone();
                let float_win_weak2 = float_win.as_weak();
                float_win.on_window_drag_started(move || {
                    if let Some(win) = float_win_weak2.upgrade() {
                        #[cfg(windows)]
                        {
                            let mut pt = windows_sys::Win32::Foundation::POINT { x: 0, y: 0 };
                            unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut pt) };
                            
                            let pos = win.window().position();
                            let offset_x = pt.x - pos.x;
                            let offset_y = pt.y - pos.y;
                            
                            *ip_clone.borrow_mut() = Some((offset_x, offset_y));
                        }
                    }
                });
                
                let ip_clone2 = initial_pos.clone();
                let float_win_weak3 = float_win.as_weak();
                let last_update_clone = last_update.clone();
                float_win.on_window_dragged(move |_, _| {
                    if last_update_clone.borrow().elapsed().as_millis() < 10 {
                        return;
                    }
                    *last_update_clone.borrow_mut() = Instant::now();

                    if let Some(win) = float_win_weak3.upgrade()
                        && let Some((offset_x, offset_y)) = *ip_clone2.borrow() {
                            #[cfg(windows)]
                            {
                                let mut pt = windows_sys::Win32::Foundation::POINT { x: 0, y: 0 };
                                unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut pt) };
                                
                                let new_x = pt.x - offset_x;
                                let new_y = pt.y - offset_y;
                                
                                use raw_window_handle::HasWindowHandle;
                                if let Ok(handle) = win.window().window_handle().window_handle()
                                    && let raw_window_handle::RawWindowHandle::Win32(w) = handle.as_raw() {
                                        let hwnd = w.hwnd.get() as *mut std::ffi::c_void;
                                        unsafe {
                                            windows_sys::Win32::UI::WindowsAndMessaging::SetWindowPos(
                                                hwnd,
                                                std::ptr::null_mut(),
                                                new_x,
                                                new_y,
                                                0,
                                                0,
                                                windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE | 
                                                windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOSIZE | 
                                                windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOZORDER |
                                                windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOCOPYBITS
                                            );
                                        }
                                    }
                            }
                            
                            #[cfg(not(windows))]
                            {
                                let pos = win.window().position();
                                win.window().set_position(slint::PhysicalPosition::new(
                                    pos.x + offset_x as i32,
                                    pos.y + offset_y as i32
                                ));
                            }
                            
                            // Slint redraw limits the event stream visually but native window moves instantly
                            win.window().request_redraw();
                        }
                });
                let ui_weak_drop = ui_weak_dock.clone();
                let title_drop = title.clone();
                let float_win_drop = float_win.as_weak();
                let tx_drop = tx_detach.clone();
                
                float_win.on_dock_dropped(move |_, _| {
                    if let Some(ui) = ui_weak_drop.upgrade()
                        && let Some(win) = float_win_drop.upgrade() {
                            let float_pos = win.window().position();
                            let float_size = win.window().size();
                            let main_pos = ui.window().position();
                            let main_size = ui.window().size();
                            
                            let overlap_x = float_pos.x < main_pos.x + main_size.width as i32 && float_pos.x + float_size.width as i32 > main_pos.x;
                            let overlap_y = float_pos.y < main_pos.y + main_size.height as i32 && float_pos.y + float_size.height as i32 > main_pos.y;
                            
                            let mut is_inside = overlap_x && overlap_y;
                            
                            if is_inside {
                                if title_drop == "Project Explorer" {
                                    ui.set_dock_project_explorer_visible(true);
                                    if ui.get_dock_left_width() < 100.0 {
                                        ui.set_dock_left_width(250.0);
                                    }
                                } else if title_drop == "Properties" {
                                    ui.set_dock_properties_visible(true);
                                    if ui.get_dock_left_width() < 100.0 {
                                        ui.set_dock_left_width(250.0);
                                    }
                                }
                                Self::save_state(&ui, &tx_drop);
                                let _ = win.hide();
                            }
                        }
                });
                
                float_win.show().unwrap();
            });
        }
    }

    fn init_splitters(&self) {
        let ui_weak_splitter_start = self.main_window.clone();
        let tx_splitter = self.layout_tx.clone();
        if let Some(ui) = ui_weak_splitter_start.upgrade() {
            let initial_splitter_width = Rc::new(RefCell::new(250.0));
            let initial_mouse_pos = Rc::new(RefCell::new(None));
            let initial_fraction = Rc::new(RefCell::new(0.5));
            
            let isw_clone = initial_splitter_width.clone();
            let imp_clone = initial_mouse_pos.clone();
            let if_clone = initial_fraction.clone();
            let ui_start = ui_weak_splitter_start.clone();
            ui.on_splitter_drag_started(move |_id| {
                if let Some(ui) = ui_start.upgrade() {
                    *isw_clone.borrow_mut() = ui.get_dock_left_width();
                    *if_clone.borrow_mut() = ui.get_splitter_fraction();
                    
                    #[cfg(windows)]
                    {
                        let mut pt = windows_sys::Win32::Foundation::POINT { x: 0, y: 0 };
                        unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut pt) };
                        *imp_clone.borrow_mut() = Some((pt.x, pt.y));
                    }
                }
            });
            
            let isw_clone2 = initial_splitter_width.clone();
            let imp_clone2 = initial_mouse_pos.clone();
            let if_clone2 = initial_fraction.clone();
            let ui_weak_splitter = ui_weak_splitter_start.clone();
            
            ui.on_splitter_dragged(move |id, _dx, _dy| {
                if let Some(ui) = ui_weak_splitter.upgrade() {
                    
                    let mut abs_dx = 0;
                    let mut abs_dy = 0;
                    
                    #[cfg(windows)]
                    {
                        if let Some((start_x, start_y)) = *imp_clone2.borrow() {
                            let mut pt = windows_sys::Win32::Foundation::POINT { x: 0, y: 0 };
                            unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut pt) };
                            abs_dx = pt.x - start_x;
                            abs_dy = pt.y - start_y;
                        }
                    }
                    
                    #[cfg(not(windows))]
                    {
                        // On non-Windows platforms we fallback to Slint's relative coords.
                        // Note: This may cause minor rubber-banding if the widget moves under the cursor.
                        abs_dx = _dx as i32;
                        abs_dy = _dy as i32;
                    }
                    
                    if id == 0 {
                        let initial = *isw_clone2.borrow();
                        let new_width = initial + (abs_dx as f32);
                        
                        if new_width < 50.0 {
                            ui.set_dock_left_width(0.0);
                        } else if new_width > 800.0 {
                            ui.set_dock_left_width(800.0);
                        } else {
                            ui.set_dock_left_width(new_width);
                        }
                    } else if id == 1 {
                        let initial_frac = *if_clone2.borrow();
                        let fraction_change = (abs_dy as f32) / 600.0;
                        let mut current_fraction = initial_frac + fraction_change;
                        current_fraction = current_fraction.clamp(0.2, 0.8);
                        ui.set_splitter_fraction(current_fraction);
                    }
                    Self::save_state(&ui, &tx_splitter);
                    ui.window().request_redraw();
                }
            });
        }
    }
}
