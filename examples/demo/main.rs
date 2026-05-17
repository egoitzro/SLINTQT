use slint::{VecModel, Model, ComponentHandle};
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use sysinfo::System;


mod mkv_parser;

use slint_qt::{CuteMainWindow, CuteTableColumn, CuteTableItem, CuteTableRowData, core, cute_connect, docking::DockManager};

fn main() -> Result<(), slint::PlatformError> {
    let ui = CuteMainWindow::new()?;

    let tray = std::sync::Arc::new(std::sync::Mutex::new(slint_qt::cute_tray::CuteSystemTray::new()));
    tray.lock().unwrap().show_message("SlintQT Demo", "¡Sistema de notificaciones nativo cargado!");

    let mut sys = System::new_all();
    sys.refresh_all();
    let cpu_name = sys.cpus().first().map(|c| c.brand()).unwrap_or("Unknown CPU");
    let total_ram_gb = sys.total_memory() as f64 / 1_073_741_824.0;
    let hardware_string = format!("CPU: {}\nRAM: {:.2} GB", cpu_name, total_ram_gb);
    ui.set_hardware_info(slint::SharedString::from(hardware_string));

    // Phase 9: Populate Stress Test (100 elements = 500 widgets)
    let stress_data: Vec<i32> = (1..=100).collect();
    ui.set_stress_items(Rc::new(VecModel::from(stress_data)).into());

    // FASE: Modelos de Datos Genéricos
    let initial_columns = vec![
        CuteTableColumn { title: "Track ID".into(), width: 140.0, data_index: 0 },
        CuteTableColumn { title: "Type".into(), width: 120.0, data_index: 1 },
        CuteTableColumn { title: "Codec & Language".into(), width: 350.0, data_index: 2 },
    ];
    let columns_model = Rc::new(VecModel::from(initial_columns));
    ui.set_table_columns(columns_model.clone().into());

    use slint_qt::CuteTreeItem;
    let initial_tree = vec![
        CuteTreeItem { id: 1, text: "Video Tracks".into(), icon: "\u{E8b2}".into(), level: 0, has_children: true, is_expanded: true, is_selected: false },
        CuteTreeItem { id: 2, text: "Track 1 (AVC/H.264)".into(), icon: "".into(), level: 1, has_children: false, is_expanded: false, is_selected: false },
        CuteTreeItem { id: 3, text: "Audio Tracks".into(), icon: "\u{E8D6}".into(), level: 0, has_children: true, is_expanded: true, is_selected: false },
        CuteTreeItem { id: 4, text: "Track 2 (AAC English)".into(), icon: "".into(), level: 1, has_children: false, is_expanded: false, is_selected: false },
        CuteTreeItem { id: 5, text: "Track 3 (AC-3 Spanish)".into(), icon: "".into(), level: 1, has_children: false, is_expanded: false, is_selected: false },
        CuteTreeItem { id: 6, text: "Subtitles".into(), icon: "\u{E8C8}".into(), level: 0, has_children: false, is_expanded: false, is_selected: false },
    ];
    let tree_model = Rc::new(VecModel::from(initial_tree));
    ui.set_tree_data(tree_model.clone().into());

    // Platform Bridge Logic: Detect OS, set theme, and hook Win32 native controls
    core::window::initialize_window(&ui);
    slint_qt::cute_vibrancy::apply_native_blur(ui.window(), false);

    // ==========================================
    // FASE 1: Motor de CuteTable (Virtual Scrolling & Multihilo)
    // ==========================================
    
    // Qt-like resize column handler
    let ui_resize_col = ui.as_weak();
    cute_connect!(ui, on_forward_column_resize, move |idx, width| {
        if let Some(ui) = ui_resize_col.upgrade() {
            let model = ui.get_table_columns();
            if let Some(mut col) = model.row_data(idx as usize) {
                // Ensure min width
                col.width = if width < 40.0 { 40.0 } else { width };
                model.set_row_data(idx as usize, col);
            }
        }
    });
    


    // ==========================================
    // FASE 2: Sistema de Delegados (CuteDelegate)
    // ==========================================
    let ui_toggled = ui.as_weak();
    cute_connect!(ui, on_delegate_toggled, move |row_idx, col_idx, val| {
        if let Some(ui) = ui_toggled.upgrade() {
            let model = ui.get_row_data();
            if let Some(mut row) = model.row_data(row_idx as usize) {
                let mut cells = row.cells.iter().collect::<Vec<_>>();
                if let Some(cell) = cells.get_mut(col_idx as usize) {
                    cell.checked = val;
                }
                row.cells = slint::ModelRc::new(slint::VecModel::from(cells));
                model.set_row_data(row_idx as usize, row);
                println!("Delegate Toggled: row={}, col={}, checked={}", row_idx, col_idx, val);
            }
        }
    });

    let ui_combo = ui.as_weak();
    cute_connect!(ui, on_delegate_combo_changed, move |row_idx, col_idx, item_idx, val| {
        if let Some(ui) = ui_combo.upgrade() {
            let model = ui.get_row_data();
            if let Some(mut row) = model.row_data(row_idx as usize) {
                let mut cells = row.cells.iter().collect::<Vec<_>>();
                if let Some(cell) = cells.get_mut(col_idx as usize) {
                    cell.combo_selected_index = item_idx;
                    // También podríamos actualizar el texto base, pero combobox ya lo maneja
                }
                row.cells = slint::ModelRc::new(slint::VecModel::from(cells));
                model.set_row_data(row_idx as usize, row);
                println!("Delegate ComboBox Changed: row={}, col={}, idx={}, val={}", row_idx, col_idx, item_idx, val);
            }
        }
    });

    // ==========================================
    // FASE 3 y 4: Sistema de Docking y Divisores (Delegado al Framework)
    // ==========================================
    let _dock_manager = DockManager::new(&ui);

    // ==========================================
    // Context Menu Integration
    // ==========================================
    let ui_ctx_menu = ui.as_weak();
    cute_connect!(ui, on_row_right_clicked_forward, move |_idx, _row| {
        if let Some(ui) = ui_ctx_menu.upgrade() {
            use slint_qt::CuteMenuItem;
            let items = vec![
                CuteMenuItem { id: 1, text: "Play Track".into(), icon: "\u{E768}".into(), has_separator: false },
                CuteMenuItem { id: 2, text: "Extract Track".into(), icon: "\u{E8C8}".into(), has_separator: true },
                CuteMenuItem { id: 3, text: "Delete".into(), icon: "\u{E74D}".into(), has_separator: false },
            ];
            ui.set_context_menu_items(Rc::new(VecModel::from(items)).into());
            ui.invoke_show_context_menu();
        }
    });

    cute_connect!(ui, on_context_menu_action, move |id| {
        println!("Context menu action triggered with ID: {}", id);
        match id {
            1 => println!("User chose to Play Track"),
            2 => println!("User chose to Extract Track"),
            3 => println!("User chose to Delete Track"),
            _ => (),
        }
    });
    // ==========================================
    // FASE: Interacción de TreeView (Virtual Scrolling)
    // ==========================================
    let ui_tree_click = ui.as_weak();
    cute_connect!(ui, on_tree_item_clicked, move |idx, mut item| {
        if let Some(ui) = ui_tree_click.upgrade() {
            let model = ui.get_tree_data();
            // Deseleccionamos todos
            for i in 0..model.row_count() {
                if let Some(mut r) = model.row_data(i)
                    && r.is_selected {
                        r.is_selected = false;
                        model.set_row_data(i, r);
                    }
            }
            // Seleccionamos el actual
            item.is_selected = true;
            model.set_row_data(idx as usize, item);
            println!("Node clicked: {}", idx);
        }
    });

    let ui_tree_toggle = ui.as_weak();
    cute_connect!(ui, on_tree_toggle_expanded, move |idx, mut item| {
        if let Some(ui) = ui_tree_toggle.upgrade() {
            let model = ui.get_tree_data();
            item.is_expanded = !item.is_expanded;
            model.set_row_data(idx as usize, item.clone());
            
            println!("Node {} expanded state changed to: {}", item.text, item.is_expanded);
            // NOTA: Para completar el efecto QTreeView, aquí se insertarían o eliminarían
            // dinámicamente los "hijos" del VecModel (Flat List Rendering).
        }
    });

    // FASE 4: Integración de Datos Reales (MKV Parser)
    // ==========================================
    let ui_add_files = ui.as_weak();
    cute_connect!(ui, on_add_source_files, move || {
        let file = rfd::FileDialog::new()
            .add_filter("Matroska", &["mkv", "mka", "mks", "mk3d"])
            .pick_file();

        if let Some(path) = file {
            let path_str = path.to_string_lossy().to_string();
            println!("Analizando archivo: {}", path_str);
            let ui_clone = ui_add_files.clone();
            
            // Hilo de Background: Analiza el archivo real sin bloquear la UI
            thread::spawn(move || {
                match mkv_parser::analyze_file(&path_str) {
                    Ok(tracks) => {
                        let mut batch = Vec::new();
                        for track in tracks {
                            let codec = track.codec;
                            let track_type = track.track_type;
                            
                            // Extraer propiedades (idioma, nombre)
                            let props = track.properties;
                            let language = props.as_ref().and_then(|p| p.language.clone()).unwrap_or_else(|| "und".to_string());
                            let name = props.as_ref().and_then(|p| p.track_name.clone()).unwrap_or_default();
                            
                            let description = if name.is_empty() { 
                                language 
                            } else { 
                                format!("{} ({})", name, language) 
                            };
                            
                            // Recolectar datos puros en el hilo, NO objetos de Slint
                            let raw_row = (
                                format!("ID {}: {}", track.id, codec),
                                track_type,
                                description
                            );
                            batch.push(raw_row);
                        }

                        // Enviamos los datos puros al hilo principal para construir los objetos Slint
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_clone.upgrade() {
                                let row_count = batch.len();
                                
                                let mut ui_batch = Vec::new();
                                for (col1, col2, col3) in batch {
                                    let cells = vec![
                                        CuteTableItem { 
                                            text: col1.into(),
                                            delegate_type: slint_qt::CuteDelegateType::Text,
                                            combo_options: slint::ModelRc::default(),
                                            combo_selected_index: 0,
                                            checked: false,
                                            progress: 0.0,
                                        },
                                        CuteTableItem { 
                                            text: col2.into(),
                                            delegate_type: slint_qt::CuteDelegateType::ComboBox,
                                            combo_options: Rc::new(VecModel::from(vec!["Video".into(), "Audio".into(), "Subtitles".into()])).into(),
                                            combo_selected_index: 0, // This should map to col2 realistically, but we just demo
                                            checked: false,
                                            progress: 0.0,
                                        },
                                        CuteTableItem { 
                                            text: col3.into(),
                                            delegate_type: slint_qt::CuteDelegateType::Text,
                                            combo_options: slint::ModelRc::default(),
                                            combo_selected_index: 0,
                                            checked: false,
                                            progress: 0.0,
                                        },
                                    ];
                                    ui_batch.push(CuteTableRowData {
                                        cells: slint::ModelRc::new(slint::VecModel::from(cells)),
                                        is_selected: false,
                                    });
                                }
                                
                                let table_model = Rc::new(VecModel::from(ui_batch));
                                ui.set_row_data(table_model.into());
                                println!("¡Tabla actualizada con {} pistas reales!", row_count);
                            }
                        }).unwrap_or_else(|e| eprintln!("EventLoop Error: {}", e));
                    }
                    Err(e) => {
                        println!("Error al analizar el archivo: {}", e);
                    }
                }
            });
        }
    });

    // --- FASE 3: PRUEBA DE CONCEPTO MUXING ---
    // Usamos el macro para conectar la señal 'start_multiplexing' al slot (closure)
    let ui_mux_handle = ui.as_weak();
    cute_connect!(ui, on_start_multiplexing, move || {
        println!("Signal received: Start Multiplexing!");
        
        // Creamos un canal de crossbeam para enviar la actualización del progreso
        let (tx, rx) = crossbeam_channel::bounded::<f32>(10);
        let ui_for_thread = ui_mux_handle.clone();

        // Hilo de procesamiento en segundo plano (El Muxer)
        thread::spawn(move || {
            for i in 1..=100 {
                thread::sleep(Duration::from_millis(30)); // Simula trabajo pesado
                let _ = tx.send(i as f32 / 100.0);
            }
        });

        // Capturamos el progreso asíncrono y actualizamos la UI (El Slot Receptor)
        thread::spawn(move || {
            while let Ok(_progress) = rx.recv() {
                let ui_handle_clone = ui_for_thread.clone();
                slint::invoke_from_event_loop(move || {
                    if let Some(_ui) = ui_handle_clone.upgrade() {
                        // Obtenemos el modelo actual
                        // Para esta demo genérica, ignoramos el progress
                        // let model = ui.get_row_data();
                        // if let Some(mut row) = model.row_data(0) {
                        //    // ... update progress
                        // }
                    }
                }).unwrap_or_else(|e| eprintln!("EventLoop Error: {}", e));
            }
            println!("Muxing complete!");
        });
    });

    // -----------------------------------------------------

    ui.run()
}
