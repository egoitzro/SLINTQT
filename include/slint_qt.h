#ifndef SLINT_QT_H
#define SLINT_QT_H

#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

namespace slint_qt {

/// Representación C-Compatible de una Celda
struct CCuteTableCell {
  const char *text;
};

/// Representación C-Compatible de una Fila
struct CCuteTableRow {
  const CCuteTableCell *cells;
  uintptr_t cell_count;
};

/// Representación C-Compatible de un elemento del Árbol (Tree View)
struct CCuteTreeItem {
  int32_t id;
  const char *text;
  const char *icon;
  int32_t level;
  bool has_children;
  bool is_expanded;
  bool is_selected;
};

using CRowClickedCallback = void(*)(uintptr_t row_index);

using CTreeItemClickedCallback = void(*)(const char *text);

using CActionTriggeredCallback = void(*)(int32_t action_id);

extern "C" {

/// Initializes the SlintQT framework.
/// Returns 0 on success.
int32_t sq_init();

/// Creates the main application window and returns an opaque pointer.
/// This acts as the primary entry point for C++.
void *sq_create_app();

/// Crea una nueva instancia de la ventana principal y la inicializa.
/// Devuelve un puntero opaco que C++ debe almacenar y pasar a otras funciones.
void *sq_window_new();

/// Libera la memoria de la ventana cuando C++ haya terminado con ella.
void sq_free_app(void *ptr);

/// Ejecuta el bucle de eventos principal (Bloqueante).
void sq_run_app(void *ptr);

/// Carga datos en la CuteTable pasados desde C++.
/// `rows` es un puntero a un array de CCuteTableRow, de longitud `row_count`.
///
/// 🛡️ **Seguridad (Rust vs C++/Qt)**:
/// En Qt (C++), actualizar un modelo desde un hilo en segundo plano usando punteros crudos
/// suele causar Data Races o Segfaults si no se usan Mutexes o el sistema de Signals/Slots correctamente.
/// Aquí, extraemos los datos crudos (Strings) en el hilo llamador, desvinculándonos de la memoria de C++
/// inmediatamente (Zero-Leak). Luego enviamos estos datos "Owned" de forma segura a través
/// de `invoke_from_event_loop`. Esto garantiza Thread-Safety (Send/Sync) por diseño del compilador.
void sq_set_table_data(void *ptr,
                       const CCuteTableRow *rows,
                       uintptr_t row_count);

/// Carga datos jerárquicos en el CuteTreeView pasados desde C++.
/// La jerarquía se define mediante la propiedad `level` de cada elemento.
void sq_set_tree_data(void *ptr, const CCuteTreeItem *items, uintptr_t item_count);

/// Registra una función C/C++ que se ejecutará cuando se haga clic en una fila.
void sq_set_row_clicked_callback(void *ptr, CRowClickedCallback callback);

/// Muestra u oculta los paneles laterales (0: Explorer, 1: Properties)
void sq_toggle_panel(void *ptr, uint32_t panel_id, bool visible);

/// Despliega un cuadro de diálogo nativo del Sistema Operativo.
/// `msg_type`: 0 = Info, 1 = Warning, 2 = Error
void sq_show_message_box(const char *title, const char *message, uint32_t msg_type);

/// Registra una función C/C++ que se ejecutará cuando se haga clic en un elemento del árbol.
void sq_set_tree_callback(void *ptr, CTreeItemClickedCallback callback);

/// Registra una función C/C++ que se ejecutará cuando se haga clic en un botón de la Toolbar.
void sq_set_action_callback(void *ptr, CActionTriggeredCallback callback);

/// Establece el texto de la barra de estado inferior.
void sq_set_status_text(void *ptr, const char *text);

/// Establece el progreso de la barra de estado inferior (de 0.0 a 1.0).
void sq_set_status_progress(void *ptr, float progress);

/// Añade una nueva pestaña y la selecciona automáticamente.
void sq_add_tab(void *ptr, const char *title);

/// Cierra una pestaña por su índice.
void sq_close_tab(void *ptr, uintptr_t index);

}  // extern "C"

}  // namespace slint_qt

#endif  // SLINT_QT_H
