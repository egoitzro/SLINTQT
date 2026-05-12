use std::ffi::{c_char, CStr, c_void};
use slint_qt::lib_ffi::{sq_init, sq_create_app};
use slint_qt::ffi::{sq_set_tree_callback, sq_run_app, sq_free_app, sq_add_tab};
use std::sync::atomic::{AtomicPtr, Ordering};

// Puntero global para poder acceder a la UI desde el callback
static APP_PTR: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

/// 🚀 Simulación de la función de callback en C++
extern "C" fn my_cpp_tree_callback(text_ptr: *const c_char) {
    if text_ptr.is_null() { return; }
    
    let c_str = unsafe { CStr::from_ptr(text_ptr) };
    if let Ok(rust_str) = c_str.to_str() {
        println!("\n[C++ Console] 🌲 ¡Clic en el árbol! Archivo: {}", rust_str);
        
        // C++: Le decimos a la UI que abra una nueva pestaña con el nombre del archivo
        let app_ptr = APP_PTR.load(Ordering::Relaxed);
        if !app_ptr.is_null() {
            println!("[C++ Console] 📄 Añadiendo nueva pestaña en la UI para: {}", rust_str);
            sq_add_tab(app_ptr, text_ptr);
        }
    }
}

/// 🚀 Simulación del manejador de la Toolbar en C++
extern "C" fn my_cpp_action_callback(action_id: i32) {
    let app_ptr = APP_PTR.load(Ordering::Relaxed);
    if app_ptr.is_null() { return; }

    use slint_qt::ffi::{sq_set_status_text, sq_set_status_progress};
    use std::ffi::CString;

    println!("\n[C++ Console] 🛠️ ¡Acción en la Toolbar! ID: {}", action_id);
    match action_id {
        0 => {
            let msg = CString::new("Preparando apertura de archivo...").unwrap();
            sq_set_status_text(app_ptr, msg.as_ptr());
            sq_set_status_progress(app_ptr, 0.2);
        },
        1 => {
            let msg = CString::new("Guardando configuración actual...").unwrap();
            sq_set_status_text(app_ptr, msg.as_ptr());
            sq_set_status_progress(app_ptr, 0.5);
        },
        3 => {
            let msg = CString::new("¡Iniciando Multiplexado MKV!").unwrap();
            sq_set_status_text(app_ptr, msg.as_ptr());
            sq_set_status_progress(app_ptr, 0.9);
            
            // Demostración Fase 17: Diálogo Nativo
            let title = CString::new("Aviso de Multiplexado").unwrap();
            let body = CString::new("Este es un diálogo nativo lanzado desde C++ usando la librería RFD de Rust. ¡Es completamente Thread-Safe!").unwrap();
            slint_qt::ffi::sq_show_message_box(title.as_ptr(), body.as_ptr(), 0);
        },
        _ => {
            let msg = CString::new("Acción genérica registrada").unwrap();
            sq_set_status_text(app_ptr, msg.as_ptr());
        }
    }
}

fn main() {
    println!("[C++ Console] Inicializando SlintQT FFI...");
    let status = sq_init();
    if status != 0 {
        eprintln!("[C++ Console] Error inicializando el motor.");
        return;
    }

    println!("[C++ Console] Creando la ventana principal...");
    let app_ptr = sq_create_app();
    if app_ptr.is_null() {
        eprintln!("[C++ Console] Fallo crítico al crear la UI.");
        return;
    }
    
    // Almacenamos el puntero globalmente para la simulación
    APP_PTR.store(app_ptr, Ordering::Relaxed);

    // Registramos nuestro callback C++ para interceptar el árbol
    println!("[C++ Console] Enganchando callback del TreeView...");
    sq_set_tree_callback(app_ptr, my_cpp_tree_callback);
    
    // Registramos el callback para la Toolbar
    println!("[C++ Console] Enganchando callback de la Toolbar...");
    slint_qt::ffi::sq_set_action_callback(app_ptr, my_cpp_action_callback);

    println!("[C++ Console] Inyectando datos jerárquicos en el TreeView...");
    use std::ffi::CString;
    use slint_qt::ffi::{CCuteTreeItem, sq_set_tree_data};
    
    let icon_folder = CString::new("\u{E8b2}").unwrap();
    let icon_file = CString::new("").unwrap();
    
    let text1 = CString::new("Carpeta de Prueba (Inyectada desde C)").unwrap();
    let text2 = CString::new("main.cpp").unwrap();
    let text3 = CString::new("build.rs").unwrap();
    
    let tree_items = vec![
        CCuteTreeItem { id: 1, text: text1.as_ptr(), icon: icon_folder.as_ptr(), level: 0, has_children: true, is_expanded: true, is_selected: false },
        CCuteTreeItem { id: 2, text: text2.as_ptr(), icon: icon_file.as_ptr(), level: 1, has_children: false, is_expanded: false, is_selected: false },
        CCuteTreeItem { id: 3, text: text3.as_ptr(), icon: icon_file.as_ptr(), level: 1, has_children: false, is_expanded: false, is_selected: false },
    ];
    
    // Al ser un vector respaldado por memoria local, es seguro pasar el raw pointer.
    // Además, sq_set_tree_data clonará los datos internamente, logrando Zero-Leak.
    unsafe { sq_set_tree_data(app_ptr, tree_items.as_ptr(), tree_items.len()) };

    // Podemos hacer una prueba apagando el panel de propiedades (panel_id = 1)
    // Descomenta la siguiente línea para probarlo:
    // sq_toggle_panel(app_ptr, 1, false);

    println!("[C++ Console] Ejecutando el Event Loop (Bloqueante)...");
    println!("👉 (Haz clic en algún elemento del árbol para ver la magia de C++)");
    sq_run_app(app_ptr);

    println!("[C++ Console] Aplicación terminada. Liberando memoria...");
    sq_free_app(app_ptr);
    println!("[C++ Console] Limpieza exitosa. ¡Adiós!");
}
