# Mandelbrot & Julia Explorer (Rust)

<p align="center">
  <img src="docs/img/1.png" width="23%">
  <img src="docs/img/2.png" width="23%">
  <img src="docs/img/3.png" width="23%">
  <img src="docs/img/4.png" width="23%">
</p>

Explorador interactivo de los conjuntos de **Mandelbrot** y **Julia** escrito en Rust.
Permite navegar por el plano complejo con zoom continuo y pan, saltar del
Mandelbrot al Julia asociado a cualquier punto, cambiar paletas de color, ajustar
el número de iteraciones, activar antialiasing, guardar capturas en PNG, salvar
y recuperar el estado de la sesión y grabar secuencias de frames para componer
vídeos.

![Estado: funcional](https://img.shields.io/badge/estado-funcional-brightgreen)
![Rust](https://img.shields.io/badge/rust-1.85%2B-orange)
![Versión](https://img.shields.io/badge/versi%C3%B3n-0.2.0-blue)

---

## Características

- **Render paralelo** con [Rayon](https://crates.io/crates/rayon): reparte las
  filas de la imagen entre todos los núcleos de la CPU automáticamente.
- **Render asíncrono en hilo aparte**: el cálculo del fractal vive en su propio
  hilo, así la ventana sigue respondiendo a teclado y ratón aunque un frame
  tarde en completarse. Los renders obsoletos se descartan en cuanto llega uno
  más nuevo.
- **Dos fractales con el mismo motor**: Mandelbrot (plano de parámetros `c`) y
  Julia (plano dinámico `z` para un `c` fijo). La iteración `z ← z² + c` es
  idéntica; solo cambia qué variable recorre el plano.
- **Salto interactivo Mandelbrot → Julia** con la tecla `J`: el punto bajo el
  cursor (o el centro si el ratón está fuera de la ventana) pasa a ser el
  parámetro `c` del Julia correspondiente.
- **Vuelta a Mandelbrot** con `M`, restaurando el último encuadre en el que
  estuviste (conserva centro y zoom).
- **Zoom continuo con la rueda del ratón**, centrado en el cursor, con factor
  multiplicativo por cada paso de rueda.
- **Zoom ×2 con click izquierdo**, centrado en el punto donde se hizo click.
- **Pan arrastrando con el botón izquierdo**: click sin arrastrar hace zoom,
  arrastrar desplaza el encuadre.
- **Vista previa durante el arrastre**: mientras se arrastra, el render se hace
  a media resolución y se escala al tamaño final, de forma que el paneo se
  siente inmediato; al soltar, se lanza el render completo.
- **Coloración suavizada** (*smooth coloring*) para evitar las bandas visibles
  típicas del fractal.
- **Detección de ciclos en Julia**: los puntos que convergen a un ciclo
  atractor se detectan comparando la órbita con la de 20 iteraciones atrás y se
  marcan como interiores sin agotar `max_iter`.
- **Antialiasing 2×2 conmutable con `A`**: 4 muestras por píxel promediadas.
- **10 paletas de color** ciclables: `Viridis`, `Inferno`, `Magma`, `Plasma`,
  `Turbo`, `Cividis`, `Hot`, `Flag`, `Twilight` y `Grayscale`.
- **Deshacer** con historial en búfer que atraviesa cambios de modo
  (Mandelbrot ↔ Julia).
- **Ajuste de iteraciones** en pasos de 1000 en caliente.
- **Guardado y carga de estado** en JSON (`F5` / `F9`): viewport, modo, `c` del
  Julia, iteraciones, paleta y estado de AA.
- **Grabación de secuencia de frames PNG** a 30 fps (`V`) para componer vídeo
  con `ffmpeg`, con nombre numerado secuencialmente.
- **Auto-zoom** con `Z`: la vista se acerca lentamente al centro de forma
  continua, pensado para grabar secuencias de zoom sin tocar el ratón.
- **Exportación a PNG** de la vista actual (con o sin leyenda).
- **Leyenda en pantalla** con modo activo, coordenadas del centro, nivel de
  zoom, iteraciones, paleta activa, estado del AA, tiempo de render, tamaño del
  historial, indicador de grabación y **coordenadas complejas del punto bajo el
  cursor** en tiempo real.

## Requisitos

### Sistema operativo

| SO | Estado | Notas |
|---|---|---|
| Linux + X11 | ✅ Soportado | Backend X11 nativo |
| Linux + Wayland | ✅ Soportado | A través de Xwayland (backend X11) |
| Windows 10/11 | ✅ Soportado | Backend Win32 (por defecto de `minifb`) |
| macOS 11+ | ✅ Soportado | Backend Cocoa (por defecto de `minifb`) |

No funciona en Wayland puro sin Xwayland activado.

### Hardware

| Componente | Mínimo | Recomendado |
|---|---|---|
| CPU | x86_64 o ARM64 | Multicore (Rayon reparte el trabajo) |
| RAM | 2 GB (1 GB libre durante la compilación) | 4 GB |
| Disco | 500 MB libres para `target/` | 1 GB |
| GPU | No necesaria (render 100 % CPU) | — |
| Pantalla | 1024×768 | 1920×1080 o superior |

### Software

- **Rust** 1.85 o superior (edición 2024), instalable vía
  [rustup](https://rustup.rs/).
- **Enlazador del sistema**:
  - Linux (Debian/Ubuntu): `sudo apt install build-essential`
  - Linux (Fedora): `sudo dnf groupinstall "Development Tools"`
  - Linux (Arch): `sudo pacman -S base-devel`
  - Windows: [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
    con la carga de trabajo "Desarrollo para el escritorio con C++".
  - macOS: Xcode Command Line Tools (`xcode-select --install`).
- **Bibliotecas de desarrollo X11** (solo Linux):
  - Debian/Ubuntu: `sudo apt install libx11-dev libxkbcommon-dev libxcb1-dev`
  - Fedora: `sudo dnf install libX11-devel libxkbcommon-devel libxcb-devel`
  - Arch: `sudo pacman -S libx11 libxkbcommon libxcb`

En la mayoría de escritorios Linux estas bibliotecas ya vienen instaladas; solo
hacen falta explícitamente en instalaciones mínimas o contenedores.

En Windows y macOS no se requiere ninguna dependencia adicional: `minifb` usa
las APIs del sistema (Win32 y Cocoa respectivamente).

## Instalación

```bash
git clone https://github.com/fserradilla65-afk/mandelbrot-rust.git
cd mandelbrot-rust
cargo build --release
```

## Uso

```bash
# Mandelbrot clásico, vista por defecto
cargo run --release

# Mandelbrot centrado en (cx, cy) con el zoom indicado
cargo run --release -- <cx> <cy> [zoom]

# Julia para c = cr + ci·i, con zoom opcional
cargo run --release -- julia <cr> <ci> [zoom]
```

## Controles

| Acción | Tecla / ratón |
|---|---|
| Zoom continuo centrado en el cursor | Rueda del ratón |
| Zoom ×2 sobre el punto | Click izquierdo |
| Pan (arrastrar la vista) | Arrastrar con botón izquierdo |
| Deshacer (undo) | Click derecho / `U` |
| **Julia desde el punto bajo el cursor** | `J` (solo en Mandelbrot) |
| **Volver al último Mandelbrot** | `M` (desde Julia) |
| Aumentar iteraciones (+1000) | `I` |
| Reducir iteraciones (−1000) | `O` |
| Cambiar paleta | `P` |
| Activar / desactivar antialiasing | `A` |
| Auto-zoom on / off | `Z` |
| Grabar / detener secuencia de frames | `V` |
| Guardar captura PNG (sin leyenda) | `S` |
| Guardar captura PNG (con leyenda) | `Shift` + `S` |
| Guardar estado de la sesión | `F5` |
| Cargar estado de la sesión | `F9` |
| Resetear al estado inicial | `R` |
| Salir | `Esc` |

### Notas sobre `J` y `M`

- **`J`** solo actúa en modo Mandelbrot. Toma las coordenadas complejas del
  píxel bajo el cursor (o del centro si el ratón está fuera de la ventana) y
  salta al `J_c` correspondiente con un encuadre inicial `[-2, 2]²`.
- **`M`** vuelve al **último encuadre de Mandelbrot** que estabas viendo, no al
  inicial. Así puedes entrar a un Julia, explorarlo y volver sin perder el
  zoom donde lo dejaste. Si arrancaste directamente en modo Julia por línea de
  comandos, `M` te lleva al Mandelbrot inicial (porque aún no había uno previo).
- **`U`** y el click derecho deshacen paso a paso, y **atraviesan cambios de
  modo**: si saltaste de Mandelbrot a Julia y luego hiciste zoom dentro del
  Julia, los sucesivos `U` recorren ese historial en orden inverso.

### Notas sobre la interacción con el ratón

- Un **click** (pulsar y soltar sin desplazar) sobre el botón izquierdo hace
  zoom ×2 sobre el punto donde se pulsó.
- Un **arrastre** (desplazar el ratón más de 3 píxeles antes de soltar) hace
  *pan*. Mientras arrastras, el render se hace a media resolución para que la
  imagen responda de inmediato; al soltar, se recalcula el frame completo.
- La **rueda** aplica un zoom multiplicativo continuo. El punto complejo bajo
  el cursor se mantiene fijo durante el zoom.

### Paletas

El ciclo con `P` recorre, en este orden:

→ Viridis → Inferno → Magma → Plasma → Turbo → Cividis → Hot → Flag → Twilight → Grayscale → (vuelve a Viridis)

- **Viridis, Inferno, Magma, Plasma, Turbo**: perceptualmente uniformes,
  seguras para daltonismo (familia Matplotlib).
- **Cividis**: también de Matplotlib, optimizada específicamente para
  daltonismo (azul → gris → amarillo).
- **Hot**: la clásica de MATLAB.
- **Flag**: 8 colores puros cíclicos, ideal para ver los bucles de iteración.
- **Twilight**: paleta cíclica, con extremos claros y centro oscuro.
- **Grayscale**: escala de grises pura.

## Capturas

Al pulsar `S` se guarda la imagen actual (sin leyenda ni barra de paleta) como
`.png` en la carpeta de imágenes del usuario:

- Linux: `~/Imágenes/Capturas/` (o `~/Pictures/Capturas/` en inglés).
- Windows: `C:\Users\<user>\Pictures\Capturas\`.
- macOS: `~/Pictures/Capturas/`.

Si se pulsa `Shift+S`, guarda la imagen **con** leyenda y barra de paleta.
El nombre del archivo incluye un timestamp en segundos y una etiqueta del tipo
de captura (`mandelbrot_<ts>_clean.png` o `mandelbrot_<ts>_legend.png`).

## Guardado y carga de estado

Con `F5` se guarda el estado actual de la sesión en un JSON dentro de la misma
carpeta `Capturas/`:

```
Capturas/mandelbrot_state.json
```

El JSON incluye:

- Viewport exacto (esquinas `x_min`, `x_max`, `y_min`, `y_max`).
- Modo actual (Mandelbrot o Julia con su parámetro `c`).
- Número de iteraciones máximo.
- Paleta activa.
- Estado del antialiasing.

Con `F9` se recupera ese estado y se re-renderiza la vista. Es útil para
volver a un punto interesante al día siguiente, o para compartir un encuadre
concreto con otra persona.

## Grabación de vídeo

Pulsando `V` se inicia la grabación de una secuencia de frames PNG a 30 fps.
Los ficheros se guardan en:

```
Capturas/frames_<timestamp>/frame_00000.png
Capturas/frames_<timestamp>/frame_00001.png
...
```

Cada frame se guarda con el mismo contenido que ves en pantalla (incluida la
leyenda). Para detener la grabación, pulsa `V` otra vez; el programa imprime
en la terminal el número total de frames y la carpeta donde quedaron.

Para montar el vídeo con `ffmpeg`:

```bash
ffmpeg -framerate 30 -i frame_%05d.png -c:v libx264 -pix_fmt yuv420p zoom.mp4
```

Combinado con `Z` (auto-zoom) puedes grabar un zoom continuo sin tocar el
ratón: pulsa `Z` para empezar el acercamiento, `V` para empezar a grabar, deja
correr, y al terminar pulsa `V` para parar y `Z` para desactivar el auto-zoom.

## Estructura del proyecto

```
mandelbrot-rust/
├── Cargo.toml
├── src/
│   ├── main.rs        # bucle de eventos y composición del frame
│   ├── cli.rs         # parseo de argumentos de línea de comandos
│   ├── fractal.rs     # FractalKind + iteración (cardioide, bulbo, ciclos)
│   ├── palette.rs     # Palette + LUT + datos de anclas RGB
│   ├── viewport.rs    # Viewport, zoom_at, pan_pixels, pixel_to_complex
│   ├── render.rs      # RenderParams + render + Renderer (hilo asíncrono)
│   ├── state.rs       # State, Snapshot, SavedState (save/load)
│   ├── ui.rs          # texto con font8x8, barra de paleta, overlay
│   └── png.rs         # guardado PNG + FrameRecorder
├── Capturas/          # Generada al pulsar S o V (no versionada)
└── README.md
```

Cada módulo tiene una responsabilidad clara y se puede probar por separado:

- **`fractal`** — `enum FractalKind { Mandelbrot, Julia { c } }` y el iterador
  `iterate`, con comprobaciones analíticas del cardioide principal y del bulbo
  de período 2, y detección de ciclos por comparación con la órbita previa.
- **`palette`** — `enum Palette` con 10 variantes, la función `color(t)` y una
  `PaletteLut` que precalcula 2048 entradas para indexar en el bucle de render.
- **`viewport`** — región del plano complejo a renderizar, con constructores
  específicos para Mandelbrot y Julia, conversión píxel ↔ complejo, zoom sobre
  un punto y desplazamiento en píxeles.
- **`render`** — `RenderParams` agrupa todo lo necesario para un frame; `render`
  lo ejecuta con Rayon; `Renderer` gestiona el hilo de fondo, la cola de
  peticiones con descarte de frames obsoletos y la recogida del resultado.
- **`state`** — todo el estado mutable del programa en un solo `struct`, con
  las transiciones (`jump_to_julia`, `back_to_mandelbrot`, `undo`, `reset`) y
  la serialización a JSON.
- **`ui`** — dibujo de caracteres con `font8x8`, composición de la leyenda,
  barra de paleta y utilidades de formato de tiempo.
- **`png`** — guardado PNG con `image` y `FrameRecorder` para secuencias.
- **`cli`** — parseo de `mandelbrot-rust [cx cy [zoom]]` y
  `mandelbrot-rust julia <cr> <ci> [zoom]`.
- **`main`** — bucle de eventos, lectura de ratón y teclado, y llamada a los
  módulos anteriores.

## Dependencias

| Crate | Versión | Uso |
|---|---|---|
| [`minifb`](https://crates.io/crates/minifb) | 0.28.0 | Ventana y bucle de eventos. En Linux se fuerza el backend X11 (`default-features = false, features = ["x11"]`); en Windows y macOS usa el backend nativo. |
| [`rayon`](https://crates.io/crates/rayon) | 1.12.0 | Paralelismo de datos en el render (reparte las filas entre todos los núcleos). |
| [`num-complex`](https://crates.io/crates/num-complex) | 0.4.6 | Tipo `Complex<f64>` para representar el parámetro `c` del Julia y para la aritmética de viewport. |
| [`image`](https://crates.io/crates/image) | 0.25.10 | Codificación PNG para las capturas (`default-features = false, features = ["png"]`). |
| [`font8x8`](https://crates.io/crates/font8x8) | 0.3.1 | Tipografía bitmap 8×8 para la leyenda en pantalla. |
| [`dirs`](https://crates.io/crates/dirs) | 5.0.1 | Resolución multiplataforma de la carpeta de imágenes del usuario. |
| [`serde`](https://crates.io/crates/serde) | 1.0 | Serialización de `Viewport`, `FractalKind`, `Palette` y `SavedState`. |
| [`serde_json`](https://crates.io/crates/serde_json) | 1.0 | Formato JSON para guardar y cargar el estado de la sesión. |

## Limitaciones conocidas

- **Wayland nativo no soportado en Linux.** La app fuerza el backend X11 de
  `minifb`, que funciona a través de Xwayland. Como consecuencia, no arranca
  en compositores Wayland sin Xwayland activado (poco habitual en escritorios
  de usuario).
- **Aviso inofensivo al cerrar (Linux/Wayland).** Al salir pueden aparecer
  mensajes `queue 0x... destroyed while proxies still attached` en la terminal.
  Son emitidos por la capa de Wayland/Xwayland al liberar recursos y no indican
  ningún error real.
- **Precisión limitada a `f64`.** Permite zooms hasta un factor de ~10¹³–10¹⁵
  según la zona explorada. Más allá aparecen imágenes cada vez más pixeladas.
  Este límite aplica tanto a Mandelbrot como a Julia. No se ha implementado
  *perturbation theory* ni doble precisión, que serían necesarios para ir más
  lejos.
- **Sin posicionamiento programático de la ventana.** `minifb` no expone API
  multiplataforma para esto. Si la ventana aparece mal colocada, muévela
  manualmente con el gestor de ventanas.
- **Cambiar de Julia requiere volver a Mandelbrot.** La tecla `J` solo actúa
  en modo Mandelbrot para evitar la ambigüedad de "¿qué `c` tomo?". Si quieres
  otro Julia, pulsa `U` (o `M`) y elige otro punto.
- **Antialiasing 2×2 opcional.** Con `A` se activa o desactiva. Con AA activo
  cada frame se calcula con 4 muestras por píxel; con AA desactivado, 1.
- **La detección de ciclos solo se aplica a Julia.** En Mandelbrot, los puntos
  interiores se detectan por cardioide/bulbo o por agotamiento de `max_iter`.

## Reconocimientos

Las paletas Viridis, Inferno, Magma, Plasma, Turbo y Cividis provienen del
proyecto Matplotlib, a su vez basadas en trabajo de Nathaniel Smith, Stéfan van
der Walt, Bastian Bechtold y otros.

Hot, Flag y Jet son herencia de MATLAB / IDL.

El conjunto de Julia y su relación con el de Mandelbrot (conexidad de `J_c` en
función de la órbita crítica de `c`) son resultados clásicos de la dinámica
compleja; véase por ejemplo *Complex Dynamics* de Carleson y Gamelin, o los
trabajos de Douady y Hubbard.

La detección analítica del cardioide principal y del bulbo de período 2 sigue
el criterio clásico descrito en la documentación de referencia del conjunto de
Mandelbrot (por ejemplo, en *The Science of Fractal Images* de Peitgen y
Saupe).

## Licencia

Este proyecto se distribuye bajo la licencia **MIT**. Consulta el archivo
[LICENSE](LICENSE) para el texto completo.

En resumen: puedes usar, copiar, modificar, fusionar, publicar, distribuir,
sublicenciar y vender copias del software, siempre que preserves el aviso de
copyright original y no se responsabilice al autor de posibles daños.
