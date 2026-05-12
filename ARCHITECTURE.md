# Slint-qt Framework Architecture

`slint-qt` is a modern, high-performance UI framework built in Rust and Slint. Its primary goal is to replicate the developer experience and modularity of the Qt Framework (specifically `QtWidgets`) while fully leveraging Windows 11 design guidelines (WinUI 3, Mica, Acrylic) and Rust's safety and concurrency.

## Completed Phases

### Fase 1: Motor de CuteTable (Virtual Scrolling & Multihilo)
- Implementation of `CuteTable` with native column resizing and virtual scrolling.
- Capable of rendering thousands of rows efficiently by delegating data models to Rust (`VecModel`).

### Fase 2: Layouts Flexibles (Splitters)
- Implementation of a native drag-to-resize `Splitter` layout.
- Synchronization of layout fractions between Slint and Rust using explicit pixel-to-fraction math.

### Fase 3: Arquitectura de Señales (Signals & Slots)
- Implementation of the `cute_connect!` macro in Rust.
- Allowed 1:1 mirroring of Qt's Signal/Slot pattern to communicate seamlessly between Slint UI threads and Rust background workers.

### Fase 4: Integración de Datos Reales (MKV Parser)
- Deep connection with standard OS CLI tools.
- `mkvmerge` background thread execution and dynamic UI updating via `slint::invoke_from_event_loop`.

### Fase 5: Capa de Tema (ThemeEngine)
- Centralized aesthetic properties via `CuteTheme`.
- Hybrid approach enforcing Light Mode internally while enabling system-level Dark/Light context.

### Fase 6 & 7: Interfaz Nativa Avanzada (TitleBar, MenuBar & Dialogs)
- Custom raw Win32 DWM manipulation for Mica backdrop and native corner radiuses.
- Injection of `HTCAPTION` and `WM_LBUTTONUP` messages to perfectly simulate native window dragging.
- Creation of `CuteMenuBar` with Acrylic `PopupWindow`s and `CuteDialog` for modal hardware/status readouts.

### Fase 8: Librería CuteWidgets (Qt Core Widgets)
- Implementation of atomic, reusable UI form components.
- Created `CuteLineEdit`, `CuteCheckBox`, `CuteComboBox`, and `CuteGroupBox`.
- Exact visual replication of Windows 11 form aesthetics with property bindings tailored for Rust (`text`, `checked`, `model`, `current_index`).

---

## Preparación para la Fase 9: Test de Estrés y Completado de Widgets

Para coronar el framework y demostrar su superioridad técnica sobre `QScrollArea` de C++, la Fase 9 consistirá en lo siguiente:

### 1. El Test de Estrés (Renderizado Masivo)
El comportamiento de instanciar cientos de *Widgets* en frameworks orientados a objetos (como Qt o WPF) tradicionalmente resulta en latencia y un consumo de RAM desproporcionado.

En esta fase:
- Renombraremos la pestaña 3 a **"Stress Test"**.
- Implementaremos un `Flickable` (el área de scroll acelerada de Slint).
- Desde Rust, inyectaremos un modelo de 100 iteraciones.
- Slint generará dinámicamente un `CuteGroupBox` por cada iteración, conteniendo múltiples instancias de nuestra librería actual (`CuteLineEdit`, `CuteCheckBox`, `CuteComboBox`).
- **Objetivo**: Superar los 500+ widgets interactivos renderizados en pantalla a 60 FPS estables sin bloquear el hilo principal.

---

## Preparación para la Fase 10: Completar el Espejo de Qt (Librería Avanzada)

Para que `slint-qt` sea un reemplazo definitivo y 1:1 de `QtWidgets`, la Fase 10 se centrará en introducir los componentes de control e información avanzados:

- **`CuteRadioButton`**: Opciones mutuamente excluyentes (esferas).
- **`CuteSlider`**: La barra deslizable para valores numéricos.
- **`CuteProgressBar`**: Una barra de progreso independiente.
- **`CuteSpinBox`**: El input numérico con las flechitas de arriba/abajo.
- **`CuteTreeView` / `CuteListView`**: Listas complejas y árboles jerárquicos (muy usados en Qt para estructuras de carpetas o dependencias de pistas).

---

## Preparación para la Fase 11: Ecosistema Completo (Framework Qt al 100%)

Dado que `slint-qt` está diseñado para ser un framework de propósito general (usando MKVToolNix solo como entorno de pruebas y estrés), la Fase 11 cubrirá todos los contenedores y variaciones que un framework de escritorio profesional exige:

1. **`CuteToolBar` y `CuteToolButton`**: Barras de herramientas modulares debajo del menú principal.
2. **`CuteStatusBar`**: Barra de estado inferior para telemetría, textos informativos y barras de progreso globales.
3. **`CuteTextEdit`**: Editor de texto multilínea con bordes redondeados y comportamiento de expansión.
4. **`CuteScrollArea`**: Envoltura nativa de scroll con barras de desplazamiento visibles e interactivas.
5. **`CuteMessageBox`**: Cajas de diálogo prefabricadas para Error/Warning/Info con botones estándar.
6. **`CuteDateTimeEdit`**: Selectores de fecha y hora.
