use std::sync::Mutex;
use std::sync::Arc;
use crate::ffi::{CRowClickedCallback, CTreeItemClickedCallback, CActionTriggeredCallback};
use std::ffi::CString;

/// 🛡️ **Seguridad (Rust vs C++/Qt)**: 
/// El patrón de Qt usualmente requiere macros MOC (`Q_OBJECT`) y el envío de señales (Signals)
/// a través de hilos que, si no se conectan con `Qt::QueuedConnection`, causan Data Races al 
/// modificar datos no protegidos.
/// En SlimCute, `SmartEngineBinding` actúa como un intermediario seguro (Thread-Safe).
/// Protegemos los callbacks C++ con `Arc<Mutex<...>>` para que puedan compartirse y
/// llamarse sin riesgo de segfaults si múltiples eventos disparan al mismo tiempo.
#[derive(Clone)]
pub struct SmartEngineBinding {
    pub on_row_clicked: Arc<Mutex<Option<CRowClickedCallback>>>,
    pub on_tree_clicked: Arc<Mutex<Option<CTreeItemClickedCallback>>>,
    pub on_action_triggered: Arc<Mutex<Option<CActionTriggeredCallback>>>,
}

impl Default for SmartEngineBinding {
    fn default() -> Self {
        Self::new()
    }
}

impl SmartEngineBinding {
    pub fn new() -> Self {
        Self {
            on_row_clicked: Arc::new(Mutex::new(None)),
            on_tree_clicked: Arc::new(Mutex::new(None)),
            on_action_triggered: Arc::new(Mutex::new(None)),
        }
    }

    pub fn register_row_clicked(&self, callback: CRowClickedCallback) {
        if let Ok(mut lock) = self.on_row_clicked.lock() {
            *lock = Some(callback);
        }
    }

    pub fn register_tree_clicked(&self, callback: CTreeItemClickedCallback) {
        if let Ok(mut lock) = self.on_tree_clicked.lock() {
            *lock = Some(callback);
        }
    }
    
    pub fn register_action_triggered(&self, callback: CActionTriggeredCallback) {
        if let Ok(mut lock) = self.on_action_triggered.lock() {
            *lock = Some(callback);
        }
    }

    pub fn emit_row_clicked(&self, row_index: usize) {
        if let Ok(lock) = self.on_row_clicked.lock() {
            if let Some(callback) = *lock {
                callback(row_index);
            }
        }
    }

    pub fn emit_tree_clicked(&self, text: &str) {
        if let Ok(lock) = self.on_tree_clicked.lock() {
            if let Some(callback) = *lock {
                // Convertimos el string de Rust a un formato seguro C-Style (terminado en nulo)
                if let Ok(c_string) = CString::new(text) {
                    callback(c_string.as_ptr());
                }
            }
        }
    }
    
    pub fn emit_action_triggered(&self, action_id: i32) {
        if let Ok(lock) = self.on_action_triggered.lock() {
            if let Some(callback) = *lock {
                callback(action_id);
            }
        }
    }
}
