use std::ffi::{CStr, c_char, c_void};
use std::rc::Rc;
use slint::{ComponentHandle, Model, VecModel};
use crate::{CuteMainWindow, CuteTableRowData, CuteTableItem, core};

pub mod smart_bindings;

/// Representación C-Compatible de una Celda
#[repr(C)]
pub struct CCuteTableCell {
    pub text: *const c_char,
}

/// Representación C-Compatible de una Fila
#[repr(C)]
pub struct CCuteTableRow {
    pub cells: *const CCuteTableCell,
    pub cell_count: usize,
}

/// Representación C-Compatible de un elemento del Árbol (Tree View)
#[repr(C)]
pub struct CCuteTreeItem {
    pub id: i32,
    pub text: *const c_char,
    pub icon: *const c_char,
    pub level: i32,
    pub has_children: bool,
    pub is_expanded: bool,
    pub is_selected: bool,
}

/// Crea una nueva instancia de la ventana principal y la inicializa.
/// Devuelve un puntero opaco que C++ debe almacenar y pasar a otras funciones.
#[unsafe(no_mangle)]
pub extern "C" fn sq_window_new() -> *mut c_void {
    match CuteMainWindow::new() {
        Ok(ui) => {
            // Inicializamos la lógica de plataforma (Mica, FFI de arrastre, etc)
            core::window::initialize_window(&ui);
            
            // Retornamos el Box convertido a puntero opaco
            Box::into_raw(Box::new(ui)) as *mut c_void
        }
        Err(err) => {
            eprintln!("Error inicializando Slint-qt: {}", err);
            std::ptr::null_mut()
        }
    }
}

/// Libera la memoria de la ventana cuando C++ haya terminado con ella.
#[unsafe(no_mangle)]
pub extern "C" fn sq_free_app(ptr: *mut c_void) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(ptr as *mut CuteMainWindow);
        }
    }
}

/// Ejecuta el bucle de eventos principal (Bloqueante).
#[unsafe(no_mangle)]
pub extern "C" fn sq_run_app(ptr: *mut c_void) {
    if ptr.is_null() { return; }
    let ui = unsafe { &*(ptr as *mut CuteMainWindow) };
    let _ = ui.run();
}

/// Carga datos en la CuteTable pasados desde C++.
/// `rows` es un puntero a un array de CCuteTableRow, de longitud `row_count`.
/// 
/// 🛡️ **Seguridad (Rust vs C++/Qt)**: 
/// En Qt (C++), actualizar un modelo desde un hilo en segundo plano usando punteros crudos 
/// suele causar Data Races o Segfaults si no se usan Mutexes o el sistema de Signals/Slots correctamente.
/// Aquí, extraemos los datos crudos (Strings) en el hilo llamador, desvinculándonos de la memoria de C++ 
/// inmediatamente (Zero-Leak). Luego enviamos estos datos "Owned" de forma segura a través 
/// de `invoke_from_event_loop`. Esto garantiza Thread-Safety (Send/Sync) por diseño del compilador.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sq_set_table_data(
    ptr: *mut c_void,
    rows: *const CCuteTableRow,
    row_count: usize,
) {
    if ptr.is_null() || rows.is_null() { return; }
    
    let ui = unsafe { &*(ptr as *mut CuteMainWindow) };
    
    // 1. Aislamos la memoria de C++ inmediatamente.
    // Convertimos los punteros C en un slice de Rust. Esto sigue siendo inseguro, 
    // pero limitamos el scope unsafe a este bloque.
    let c_rows = unsafe { std::slice::from_raw_parts(rows, row_count) };
    
    let mut raw_rows = Vec::with_capacity(row_count);
    
    for c_row in c_rows {
        let c_cells = unsafe { std::slice::from_raw_parts(c_row.cells, c_row.cell_count) };
        let mut string_cells = Vec::with_capacity(c_row.cell_count);
        
        for c_cell in c_cells {
            let text = if c_cell.text.is_null() {
                String::new()
            } else {
                // to_string_lossy maneja caracteres inválidos UTF-8 sin crashear, 
                // a diferencia de QString::fromUtf8() que puede fallar o mostrar basura.
                unsafe { CStr::from_ptr(c_cell.text).to_string_lossy().into_owned() }
            };
            string_cells.push(text);
        }
        raw_rows.push(string_cells);
    }
    
    // 2. Transmisión Thread-Safe al Event Loop.
    // closure `move` transfiere el Ownership de raw_rows al hilo de UI.
    // Es imposible que C++ modifique estos datos mientras se renderizan (Elimina Data Races).
    let ui_weak = ui.as_weak();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let mut ui_batch = Vec::with_capacity(raw_rows.len());
            for string_cells in raw_rows {
                let slint_cells = string_cells
                    .into_iter()
                    .map(|text| CuteTableItem { 
                        text: text.into(),
                        delegate_type: crate::CuteDelegateType::Text,
                        combo_options: slint::ModelRc::default(),
                        combo_selected_index: 0,
                        checked: false,
                        progress: 0.0,
                    })
                    .collect::<Vec<_>>();
                    
                ui_batch.push(CuteTableRowData {
                    cells: slint::ModelRc::new(slint::VecModel::from(slint_cells)),
                    is_selected: false,
                });
            }
            let table_model = Rc::new(VecModel::from(ui_batch));
            ui.set_row_data(table_model.into());
        }
    });
}

/// Carga datos jerárquicos en el CuteTreeView pasados desde C++.
/// La jerarquía se define mediante la propiedad `level` de cada elemento.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sq_set_tree_data(
    ptr: *mut c_void,
    items: *const CCuteTreeItem,
    item_count: usize,
) {
    if ptr.is_null() || items.is_null() { return; }
    
    let ui = unsafe { &*(ptr as *mut CuteMainWindow) };
    let c_items = unsafe { std::slice::from_raw_parts(items, item_count) };
    
    let mut safe_items = Vec::with_capacity(item_count);
    
    for c_item in c_items {
        let text_str = if c_item.text.is_null() {
            String::new()
        } else {
            unsafe { CStr::from_ptr(c_item.text).to_string_lossy().into_owned() }
        };
        
        let icon_str = if c_item.icon.is_null() {
            String::new()
        } else {
            unsafe { CStr::from_ptr(c_item.icon).to_string_lossy().into_owned() }
        };
        
        safe_items.push((
            c_item.id,
            text_str,
            icon_str,
            c_item.level,
            c_item.has_children,
            c_item.is_expanded,
            c_item.is_selected,
        ));
    }
    
    let ui_weak = ui.as_weak();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let slint_items = safe_items.into_iter().map(|(id, text, icon, level, has_children, is_expanded, is_selected)| {
                crate::CuteTreeItem {
                    id,
                    text: text.into(),
                    icon: icon.into(),
                    level,
                    has_children,
                    is_expanded,
                    is_selected,
                }
            }).collect::<Vec<_>>();
            
            ui.set_tree_data(slint::ModelRc::new(slint::VecModel::from(slint_items)));
        }
    });
}

// ==========================================
// CALLBACKS (De Rust a C++)
// ==========================================

pub type CRowClickedCallback = extern "C" fn(row_index: usize);

/// Registra una función C/C++ que se ejecutará cuando se haga clic en una fila.
#[unsafe(no_mangle)]
pub extern "C" fn sq_set_row_clicked_callback(
    ptr: *mut c_void,
    callback: CRowClickedCallback,
) {
    if ptr.is_null() { return; }
    let ui = unsafe { &*(ptr as *mut CuteMainWindow) };
    
    // Instanciamos el binding seguro
    let engine_binding = smart_bindings::SmartEngineBinding::new();
    engine_binding.register_row_clicked(callback);
    
    ui.on_row_right_clicked_forward(move |idx, _row| {
        // Disparamos usando la capa de seguridad en lugar de llamar el raw pointer
        engine_binding.emit_row_clicked(idx as usize);
    });
}

pub type CTreeItemClickedCallback = extern "C" fn(text: *const c_char);

/// Muestra u oculta los paneles laterales (0: Explorer, 1: Properties)
#[unsafe(no_mangle)]
pub extern "C" fn sq_toggle_panel(
    ptr: *mut c_void,
    panel_id: u32,
    visible: bool,
) {
    if ptr.is_null() { return; }
    let ui_weak = unsafe { (*(ptr as *mut CuteMainWindow)).as_weak() };
    
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            if panel_id == 0 {
                ui.set_dock_project_explorer_visible(visible);
            } else if panel_id == 1 {
                ui.set_dock_properties_visible(visible);
            }
        }
    });
}

/// Despliega un cuadro de diálogo nativo del Sistema Operativo.
/// `msg_type`: 0 = Info, 1 = Warning, 2 = Error
#[unsafe(no_mangle)]
pub extern "C" fn sq_show_message_box(
    title: *const c_char,
    message: *const c_char,
    msg_type: u32,
) {
    if title.is_null() || message.is_null() { return; }
    
    let title_str = unsafe { std::ffi::CStr::from_ptr(title) }.to_string_lossy().into_owned();
    let msg_str = unsafe { std::ffi::CStr::from_ptr(message) }.to_string_lossy().into_owned();
    
    let rfd_level = match msg_type {
        1 => rfd::MessageLevel::Warning,
        2 => rfd::MessageLevel::Error,
        _ => rfd::MessageLevel::Info,
    };
    
    // Mostramos el diálogo nativo (bloqueante para el hilo que lo invoca)
    rfd::MessageDialog::new()
        .set_title(&title_str)
        .set_description(&msg_str)
        .set_level(rfd_level)
        .show();
}
/// Registra una función C/C++ que se ejecutará cuando se haga clic en un elemento del árbol.
#[unsafe(no_mangle)]
pub extern "C" fn sq_set_tree_callback(
    ptr: *mut c_void,
    callback: CTreeItemClickedCallback,
) {
    if ptr.is_null() { return; }
    let ui = unsafe { &*(ptr as *mut CuteMainWindow) };
    
    // Instanciamos el binding seguro
    let engine_binding = smart_bindings::SmartEngineBinding::new();
    engine_binding.register_tree_clicked(callback);
    
    ui.on_tree_item_clicked(move |_idx, item| {
        // Disparamos usando la capa de seguridad en lugar de llamar el raw pointer
        let text: String = item.text.into();
        engine_binding.emit_tree_clicked(&text);
    });
}

pub type CActionTriggeredCallback = extern "C" fn(action_id: i32);

/// Registra una función C/C++ que se ejecutará cuando se haga clic en un botón de la Toolbar.
#[unsafe(no_mangle)]
pub extern "C" fn sq_set_action_callback(
    ptr: *mut c_void,
    callback: CActionTriggeredCallback,
) {
    if ptr.is_null() { return; }
    let ui = unsafe { &*(ptr as *mut CuteMainWindow) };
    
    let engine_binding = smart_bindings::SmartEngineBinding::new();
    engine_binding.register_action_triggered(callback);
    
    let b1 = engine_binding.clone();
    ui.on_action_triggered(move |action_id| {
        b1.emit_action_triggered(action_id);
    });
    
    // Mapeamos también los eventos clásicos de la interfaz al Actions FFI
    let b2 = engine_binding.clone();
    ui.on_add_source_files(move || {
        b2.emit_action_triggered(0); // 0 = Open
    });
    
    let b3 = engine_binding.clone();
    ui.on_start_multiplexing(move || {
        b3.emit_action_triggered(3); // 3 = Start Multiplexing
    });
}

/// Establece el texto de la barra de estado inferior.
#[unsafe(no_mangle)]
pub extern "C" fn sq_set_status_text(
    ptr: *mut c_void,
    text: *const c_char,
) {
    if ptr.is_null() || text.is_null() { return; }
    
    let ui_weak = unsafe { (*(ptr as *mut CuteMainWindow)).as_weak() };
    let c_str = unsafe { std::ffi::CStr::from_ptr(text) };
    let text_str = c_str.to_string_lossy().into_owned();
    
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_status_text(text_str.into());
        }
    });
}

/// Establece el progreso de la barra de estado inferior (de 0.0 a 1.0).
#[unsafe(no_mangle)]
pub extern "C" fn sq_set_status_progress(
    ptr: *mut c_void,
    progress: f32,
) {
    if ptr.is_null() { return; }
    let ui_weak = unsafe { (*(ptr as *mut CuteMainWindow)).as_weak() };
    
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_status_progress(progress);
        }
    });
}

/// Añade una nueva pestaña y la selecciona automáticamente.
#[unsafe(no_mangle)]
pub extern "C" fn sq_add_tab(
    ptr: *mut c_void,
    title: *const c_char,
) {
    if ptr.is_null() || title.is_null() { return; }
    
    let ui_weak = unsafe { (*(ptr as *mut CuteMainWindow)).as_weak() };
    let c_str = unsafe { std::ffi::CStr::from_ptr(title) };
    let text = match c_str.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return,
    };
    
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let mut tabs: Vec<slint::SharedString> = ui.get_tabs().iter().collect();
            tabs.push(text.into());
            let new_idx = tabs.len() - 1;
            ui.set_tabs(slint::ModelRc::new(slint::VecModel::from(tabs)));
            ui.set_active_tab(new_idx as i32);
        }
    });
}

/// Cierra una pestaña por su índice.
#[unsafe(no_mangle)]
pub extern "C" fn sq_close_tab(
    ptr: *mut c_void,
    index: usize,
) {
    if ptr.is_null() { return; }
    
    let ui_weak = unsafe { (*(ptr as *mut CuteMainWindow)).as_weak() };
    
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let model = ui.get_tabs();
            if index < model.row_count() {
                let mut tabs: Vec<slint::SharedString> = Vec::with_capacity(model.row_count() - 1);
                for i in 0..model.row_count() {
                    if i != index {
                        if let Some(t) = model.row_data(i) {
                            tabs.push(t);
                        }
                    }
                }
                ui.set_tabs(slint::ModelRc::new(slint::VecModel::from(tabs)));
                // Ajustar pestaña activa
                let current_active = ui.get_active_tab();
                if current_active as usize == index {
                    if current_active > 0 {
                        ui.set_active_tab(current_active - 1);
                    }
                } else if (current_active as usize) > index {
                    ui.set_active_tab(current_active - 1);
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn test_ffi_null_pointer_safety() {
        // Aseguramos que pasar punteros nulos no cause un panic o segfault.
        sq_free_app(ptr::null_mut());
        sq_run_app(ptr::null_mut());
        unsafe {
            sq_set_table_data(ptr::null_mut(), ptr::null(), 0);
        }
        
        let dummy_row = CCuteTableRow { cells: ptr::null(), cell_count: 0 };
        unsafe {
            sq_set_table_data(ptr::null_mut(), &dummy_row, 1);
        }
        
        sq_set_row_clicked_callback(ptr::null_mut(), dummy_callback);
    }
    
    extern "C" fn dummy_callback(_: usize) {}
    
    #[test]
    fn test_smart_bindings_safety() {
        // Pruebas concurrentes para validar Thread-Safety y Mutex
        use std::thread;
        let binding = smart_bindings::SmartEngineBinding::new();
        let binding_clone = binding.clone();
        
        let handle = thread::spawn(move || {
            binding_clone.register_row_clicked(dummy_callback);
            binding_clone.emit_row_clicked(42);
        });
        
        binding.emit_row_clicked(10); // Ejecutar en paralelo
        assert!(handle.join().is_ok());
    }
}
