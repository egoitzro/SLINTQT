# SlintQT - Arquitectura del DockManager

El `DockManager` es el núcleo responsable de orquestar el comportamiento de las ventanas desacoplables (Docking) y los divisores dinámicos (Splitters) en el framework SlintQT.

## 1. Encapsulación en el Framework
A diferencia de implementaciones triviales donde la lógica recae en el programa del usuario final (ej: `examples/demo`), en SlintQT toda esta matemática y gestión de estado está encapsulada dentro de la propia librería:
- **Archivo Principal**: `src/docking.rs`
- **Integración**: Se inyecta en el ciclo de vida del usuario con una sola línea: `let _dock_manager = DockManager::new(&ui);`

## 2. Gestión de Ventanas Flotantes (Floating Windows)
Cuando un usuario arrastra la cabecera de un panel acoplado, el `DockManager` reacciona:
1. Extrae el título del panel.
2. Oculta el panel dentro de la ventana principal (`CuteMainWindow`).
3. Instancia dinámicamente un `CuteFloatingWindow` (ventana borderless separada).
4. Sincroniza el modelo de datos (inyecta las variables reactivas como los `tree_data`) hacia la nueva ventana.
5. Inyecta delegados de comunicación para que cualquier interacción dentro de la ventana flotante (clicks en árboles de datos, etc) se propague al hilo principal de la aplicación.

## 3. Movimiento Nativo y Renderizado de Alta Frecuencia
El arrastre de ventanas flotantes sin borde (frameless) en Slint generaba inicialmente problemas de desincronización por el cálculo de coordenadas relativas (dx, dy).

Se ha resuelto integrando la **API de Windows (Win32)**:
- Se capturan las coordenadas absolutas de la pantalla directamente desde el Kernel de Windows usando `GetCursorPos` en el evento `window_drag_started`.
- Durante el evento continuo `window_dragged`, en lugar de calcular movimientos relativos que se solapan en tiempo, se calcula la distancia absoluta y se transpone usando `SetWindowPos` con el flag `SWP_NOCOPYBITS`.
- Este enfoque permite mantener vivo el ciclo de eventos del ratón (Mouse Up viaja sin ser interceptado por un secuestro modal nativo), permitiendo que elementos dentro de la cabecera como el Botón Cerrar (X) sigan interactuables.

### Soporte Multiplataforma (macOS / Linux)
Toda la integración de bajo nivel con la API de Windows está protegida mediante directivas del compilador (`#[cfg(windows)]`). 
En sistemas operativos **macOS y Linux** (`#[cfg(not(windows))]`), el framework detecta automáticamente la plataforma y hace uso de un *fallback* seguro utilizando la abstracción estándar de Slint (`win.window().set_position(...)`). Esto garantiza que el código compile y el *Dynamic Docking* funcione en cualquier plataforma sin requerir modificaciones en tu código.

## 4. Próximos Pasos (Hoja de Ruta)
* **Snap Zones**: Detección geométrica de colisión (Hitboxes) entre la ventana flotante en arrastre y la ventana principal para activar el retorno al dock (Drop Zones).
* **Persistencia de Layout**: Serialización del estado de los paneles y los Splitters a un `layout.json` para restaurar el entorno de trabajo del usuario.
