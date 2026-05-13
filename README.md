# SlintQT Framework

**SlintQT** es una capa de infraestructura avanzada desarrollada sobre [Slint](https://slint.dev/). Su misión es superar a Qt en **fiabilidad, seguridad de memoria y facilidad de uso**, utilizando Rust como motor principal.

El objetivo de SlintQT no es construir una aplicación final, sino proporcionar un motor y un sistema de abstracciones robusto para que otras aplicaciones (incluso motores heredados en C++) puedan gozar de interfaces modernas sin el riesgo de fugas de memoria o Data Races.

## 🛡️ Zero-Leak Architecture & Seguridad

SlintQT elimina clases enteras de bugs comunes en Qt (C++) mediante el uso estricto del Ownership Model y Borrow Checker de Rust:

- **Sin `QObject` Dangling Pointers**: En C++, cuando un `QObject` padre es destruido, sus hijos se destruyen. Si una hebra o un evento intenta acceder a un hijo, ocurre un Segfault. En SlintQT, los modelos de interfaz (ej. `VecModel`) están envueltos en `Rc` (Reference Counting) en el hilo principal, garantizando que los datos visuales vivan exactamente el tiempo necesario.
- **Smart Bindings (Thread-Safety)**: En Qt, emitir una señal desde un hilo secundario sin usar `Qt::QueuedConnection` causa corrupción de datos. SlintQT utiliza `slint::invoke_from_event_loop` y Mutexes (`Arc<Mutex<T>>`) en su capa FFI. Si un motor en C++ intenta inyectar datos desde un worker thread, el FFI de SlintQT clona los datos crudos, aísla la memoria insegura, y transfiere el Ownership (Send/Sync) de manera segura al hilo de renderizado.
- **Abstracción de FFI Segura**: El programador final en Rust **jamás ve código `unsafe`**. El módulo `smart_bindings` actúa como un escudo: el desarrollador simplemente usa cierres (closures) limpios en Rust, mientras la capa inferior gestiona el ciclo de vida de los punteros C-ABI.

## ⚡ Performance Benchmarking & Renderizado Optimizado

A diferencia del pipeline de QGraphicsView o QWidget que usa la CPU intensivamente para trazar primitivas, SlintQT aprovecha que Slint compila a código nativo fuertemente tipado:
- **Zero-Copy y Lazy Evaluation**: Sugerimos minimizar el envío de Strings gigantes. El framework procesa celdas mediante vistas (`&str` convertidos en tiempo de inyección) y Slint sólo renderiza los elementos visibles (Virtual Scrolling implementado nativamente en el `ListView` de `CuteTable`).
- **GPU Acceleration**: Todo el dibujo es derivado a Skia/FemtoVG. La CPU se minimiza asegurando que la UI sólo se actualiza a través de deltas.

## 🎯 Arquitectura Principal

### 1. El Motor Core (`src/core`)
Intercepta la ventana generada por Slint y le inyecta integraciones nativas profundas utilizando `raw-window-handle`:
- **Windows 11**: Efecto **Mica**, bordes redondeados nativos (`DWMWCP_ROUND`), y gestión de áreas de arrastre (`HTCAPTION`).
- **MacOS / Linux**: Detección automática del SO (`cfg(target_os)`). Cede el control de decoración a los gestores de ventanas nativos para respetar la ergonomía del usuario.

### 2. CuteWidgets (`ui/`)
Componentes "Caja Negra" diseñados para ser agnósticos de los datos, e inyectables desde el backend:
- **`CuteTable`**: Columnas redimensionables, anchos dinámicos, virtualización de vistas (Virtual Scrolling).
- **`CuteContextMenu`**: Sistema de menús asíncronos basado en `PopupWindow`.

### 3. Smart Bindings FFI (`src/ffi/`)
La capa de interoperabilidad. Expone un C-ABI estable que permite a proyectos como KDE o MKVToolNix conectarse a la interfaz sin reescribir su núcleo en C++, beneficiándose automáticamente de la barrera de seguridad de SlintQT.

## 🚀 Cómo ejecutar la Demo

Todo código relacionado con un caso de uso específico (como nuestra aplicación original de Multiplexado de video) ha sido refactorizado y extraído al directorio `examples/`.

Para ejecutar la demostración completa de las capacidades del framework:

```bash
cargo run --example demo
```

La demo incluye:
- Parseo real asíncrono en un hilo de fondo.
- Inyección de datos multi-hilo a la interfaz usando `slint::invoke_from_event_loop` (thread-safety).
- Resizing de columnas con callback de Rust.
- Click derecho en la tabla para invocar el Menú Contextual Genérico.

## 📦 Uso como Librería

Slint-qt compila de manera predeterminada como `rlib` para proyectos nativos en Rust y como `cdylib` para integraciones FFI.

```toml
# En el Cargo.toml de tu proyecto final
[dependencies]
slint-qt = { path = "../slint-qt" }
```

Y en tu lógica de aplicación:
```rust
use slint_qt::{CuteMainWindow, core};

fn main() -> Result<(), slint::PlatformError> {
    let ui = CuteMainWindow::new()?;
    
    // Inyección de infraestructura multiplataforma
    core::window::initialize_window(&ui);
    
    // ... Tu lógica de negocio y señales aquí ...

    ui.run()
}
```

## 🛠 Próximos Pasos (Hoja de Ruta)
- Consolidación del C-ABI para enviar modelos de árbol (`QTreeWidget` equivalente) a través del FFI.
- Implementación de un `CuteTreeView` con soporte de anidamiento y colapso de nodos.
- Gestión avanzada de `PlatformTheme` para reaccionar a los cambios de "Modo Oscuro/Claro" del SO en tiempo real.
