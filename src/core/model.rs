use std::rc::Rc;
use slint::{Model, ModelTracker, ModelNotify, ModelRc, VecModel};
use crate::{CuteTableRowData, CuteTableItem, CuteDelegateType};

/// Roles de datos equivalentes a Qt::ItemDataRole
pub enum ItemRole {
    Display,       // Texto a mostrar
    Decoration,    // Iconos o Colores
    CheckState,    // Estado del Checkbox
}

/// Equivalente a QVariant en C++
pub enum ItemData {
    Text(String),
    Icon(String),
    Checked(bool),
    Empty,
}

/// Este es el Trait puro en Rust (El equivalente exacto a QAbstractItemModel).
pub trait AbstractTableModel {
    fn row_count(&self) -> usize;
    fn column_count(&self) -> usize;
    
    /// Devuelve los datos para una celda específica dependiendo del Rol solicitado.
    /// Esto permite pasar iconos, estados booleanos y texto sin clonar de más.
    fn data(&self, row: usize, column: usize, role: ItemRole) -> ItemData;
}

/// Adaptador ("El Dispatcher") que conecta nuestro Trait con Slint.
/// NOTA: Esta estructura vive en el hilo de la UI (Event Loop).
/// Si el backend de Rust o C++ quiere notificar un cambio desde otro hilo, 
/// DEBE usar `slint::invoke_from_event_loop` para acceder a este adaptador.
pub struct SlintTableModelAdapter<T: AbstractTableModel> {
    backend: Rc<T>,
    notify: ModelNotify,
}

impl<T: AbstractTableModel> SlintTableModelAdapter<T> {
    pub fn new(backend: Rc<T>) -> Self {
        Self {
            backend,
            notify: ModelNotify::default(),
        }
    }
    
    /// Señal nativa (Dispatcher): Dispara un redibujado en la capa WGPU 
    /// únicamente para esta fila. (Debe llamarse desde el Hilo Principal).
    pub fn emit_row_changed(&self, row: usize) {
        self.notify.row_changed(row);
    }
    
    pub fn emit_rows_added(&self, index: usize, count: usize) {
        self.notify.row_added(index, count);
    }
    
    pub fn emit_rows_removed(&self, index: usize, count: usize) {
        self.notify.row_removed(index, count);
    }
    
    /// Equivalente a emit_layout_changed() o layoutChanged() en Qt.
    /// Útil para ráfagas masivas (Batch Updates)
    pub fn emit_model_reset(&self) {
        self.notify.reset();
    }
}


/// Al implementar `slint::Model`, Slint automatically sabe cómo hacer 
/// "Virtual Scrolling" con nuestro adaptador. No se clonará el array completo, 
/// Slint solo pedirá `row_data` a medida que el usuario hace scroll.
impl<T: AbstractTableModel> Model for SlintTableModelAdapter<T> {
    type Data = CuteTableRowData;

    fn row_count(&self) -> usize {
        self.backend.row_count()
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        if row >= self.backend.row_count() {
            return None;
        }
        
        let cols = self.backend.column_count();
        let mut cells = Vec::with_capacity(cols);
        
        // Consultamos al backend en tiempo real (Virtualización extrema)
        for c in 0..cols {
            // Requerimos el rol Display por defecto para la celda principal
            let data = self.backend.data(row, c, ItemRole::Display);
            let text = match data {
                ItemData::Text(s) => s,
                _ => String::new(),
            };
            
            cells.push(CuteTableItem {
                text: text.into(),
                delegate_type: CuteDelegateType::Text,
                combo_options: ModelRc::default(),
                combo_selected_index: 0,
                checked: false,
                progress: 0.0,
            });
        }
        
        Some(CuteTableRowData {
            cells: ModelRc::new(VecModel::from(cells)),
            is_selected: false,
        })
    }

    // Vinculamos nuestro Dispatcher nativo con el tracker de Slint
    fn model_tracker(&self) -> &dyn ModelTracker {
        &self.notify
    }
}
