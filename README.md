# Mandelbrot & Julia Explorer (Rust)

<p align="center">
  <img src="docs/img/1.png" width="23%">
  <img src="docs/img/2.png" width="23%">
  <img src="docs/img/3.png" width="23%">
  <img src="docs/img/4.png" width="23%">
</p>

Explorador interactivo de los conjuntos de **Mandelbrot** y **Julia** escrito en Rust.
Renderiza el fractal en tiempo real, permite hacer zoom interactivo, saltar del
Mandelbrot al Julia asociado a cualquier punto, cambiar paletas de color, ajustar
el número de iteraciones y guardar capturas en PNG.

![Estado: funcional](https://img.shields.io/badge/estado-funcional-brightgreen)
![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)

---

## Características

- **Render paralelo** con [Rayon](https://crates.io/crates/rayon): usa todos los núcleos
  de tu CPU automáticamente.
- **Dos fractales con el mismo motor**: Mandelbrot (plano de parámetros `c`) y
  Julia (plano dinámico `z` para un `c` fijo). La iteración `z ← z² + c` es idéntica;
  solo cambia qué variable recorre el plano.
- **Salto interactivo Mandelbrot → Julia** con la tecla `J`: el punto bajo el cursor
  (o el centro si no hay ratón) pasa a ser el parámetro `c` del Julia correspondiente.
- **Vuelta a Mandelbrot** con `M`, **restaurando el último encuadre** en el que
  estuviste (conserva centro y zoom).
- **Coloración suavizada** (*smooth coloring*) para evitar las bandas visibles típicas
  del fractal.
- **8 paletas de color** ciclables, inspiradas en las de Matplotlib:
  `Grayscale`, `Viridis`, `Inferno`, `Plasma`, `Turbo`, `Hot`, `Flag`, `Twilight`.
- **Zoom interactivo** con un simple toque de ratón.
- **Deshacer zoom** con historial en buffer, que además atraviesa cambios de modo
  (Mandelbrot ↔ Julia).
- **Ajuste de iteraciones** en pasos de 1000 en tiempo real.
- **Exportación a PNG** de la vista actual (con o sin leyenda).
- **Leyenda en pantalla** con modo activo, coordenadas del centro, nivel de zoom,
  iteraciones, paleta activa, tiempo de render y controles.

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
| CPU | x86_64 o ARM64, 1 núcleo | 4+ núcleos (Rayon reparte el trabajo) |
| RAM | 2 GB (1 GB libre durante la compilación) | 4 GB |
| Disco | 500 MB libres para `target/` | 1 GB |
| GPU | No necesaria (render 100 % CPU) | — |
| Pantalla | 1024×768 | 1920×1080 o superior |

Con 1 solo núcleo el programa funciona, pero cada render puede tardar varios
segundos. A partir de 4 núcleos la interacción es fluida.

### Software

- **Rust** 1.75 o superior, instalable vía [rustup](https://rustup.rs/).
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

La primera compilación tarda un poco (sobre todo por la dependencia `image`),
pero las siguientes son casi instantáneas.

## Uso

```bash
# Mandelbrot clásico, vista por defecto
cargo run --release

# Mandelbrot centrado en (cx, cy) con el zoom indicado
cargo run --release -- <cx> <cy> [zoom]

# Julia para c = cr + ci·i, con zoom opcional
cargo run --release -- julia <cr> <ci> [zoom]
```

Ejemplos:

```bash
# Mandelbrot en el valle de los caballitos de mar
cargo run --release -- -0.743 0.131 100

# Julia del conejo de Douady
cargo run --release -- julia -0.123 0.745

# Julia de la basílica, con zoom 2
cargo run --release -- julia -1.0 0.0 2
```

Usa **siempre `--release`**: en modo debug el render puede ser 20–50 veces más lento.

## Controles

| Acción | Tecla / ratón |
|---|---|
| Zoom (×2) sobre el punto | Click izquierdo |
| Deshacer (undo) | Click derecho / `U` |
| **Julia desde el punto bajo el cursor** | `J` (solo en Mandelbrot) |
| **Volver al último Mandelbrot** | `M` (desde Julia) |
| Aumentar iteraciones (+1000) | `I` |
| Reducir iteraciones (−1000) | `O` |
| Cambiar paleta | `P` |
| Guardar captura PNG (sin leyenda) | `S` |
| Guardar captura PNG (con leyenda) | `Shift`+`S` |
| Resetear al estado inicial | `R` |
| Salir | `Esc` |

### Notas sobre `J` y `M`

- **`J`** solo actúa en modo Mandelbrot. Toma las coordenadas complejas del píxel
  bajo el cursor (o del centro si el ratón está fuera de la ventana) y salta al
  `J_c` correspondiente con un encuadre inicial `[-2, 2]²`.
- **`M`** vuelve al **último encuadre de Mandelbrot** que estabas viendo, no al
  inicial. Así puedes entrar a un Julia, explorarlo y volver sin perder el zoom
  donde lo dejaste. Si arrancaste directamente en modo Julia por línea de comandos,
  `M` te lleva al Mandelbrot inicial (porque aún no había uno previo).
- **`U`** y el click derecho deshacen paso a paso, y **atraviesan cambios de modo**:
  si saltaste de Mandelbrot a Julia y luego hiciste zoom dentro del Julia, los
  sucesivos `U` recorren ese historial en orden inverso.

### Paletas

El ciclo con `P` recorre, en este orden:

→ Viridis → Inferno → Plasma → Turbo → Hot → Flag → Twilight → Grayscale

- **Viridis, Inferno, Plasma, Turbo**: perceptualmente uniformes, seguras para
  daltonismo (familia Matplotlib).
- **Hot**: la clásica de MATLAB.
- **Flag**: 8 colores puros cíclicos, ideal para ver los bucles de iteración.
- **Twilight**: paleta cíclica, con extremos claros y centro oscuro.
- **Grayscale**: escala de grises pura.

### Capturas

Al pulsar `S` se guarda la imagen actual (sin leyenda ni barra de paleta) como
`.png` en la carpeta de imágenes del usuario:

- Linux: `~/Imágenes/Capturas/` (o `~/Pictures/Capturas/` en inglés).
- Windows: `C:\Users\<user>\Pictures\Capturas\`.
- macOS: `~/Pictures/Capturas/`.

Si se pulsa `Shift+S`, guarda la imagen **con** leyenda y barra de paleta.
El nombre del archivo incluye un timestamp en segundos (`mandelbrot_<ts>.png`).

## Estructura del proyecto

```
mandelbrot-rust/
├── Cargo.toml
├── src/
│   └── main.rs        # Todo el programa (modos, paletas, render, UI, capturas)
├── Capturas/          # Generada al pulsar S (no versionada)
└── README.md
```

El programa es un único archivo `main.rs`, organizado en secciones:

- **Modo de fractal** — `enum FractalKind { Mandelbrot, Julia { c } }` y su
  nombre para la leyenda.
- **Paletas de color** — `enum Palette` + datos de anclas RGB.
- **Viewport** — región del plano complejo a renderizar, con constructores
  específicos para Mandelbrot (`[-2, 1] × [-1.5, 1.5]`) y Julia (`[-2, 2]²`).
- **Render** — cálculo del fractal con Rayon y *smooth coloring*; el mismo
  bucle sirve a ambos modos cambiando la inicialización `(z₀, c)`.
- **Zoom** — transformación de coordenadas pantalla → plano complejo.
- **Texto y leyenda** — rectángulo invertido, tipografía `font8x8`, formato de
  tiempo `hh:mm:ss:cs`.
- **Capturas** — conversión `0x00RRGGBB` → RGB8 y guardado con `image`.
- **Argumentos** — parseo de CLI (`mandelbrot-rust [cx cy [zoom]]` /
  `mandelbrot-rust julia <cr> <ci> [zoom]`).
- **main** — bucle de eventos, manejo de ratón y teclado, historial de undo
  que incluye el modo, memoria del último Mandelbrot, composición del frame.

## Dependencias

El proyecto usa un conjunto reducido de crates, todos ellos maduros y multiplataforma:

| Crate | Versión | Uso |
|---|---|---|
| [`minifb`](https://crates.io/crates/minifb) | 0.28.0 | Ventana y bucle de eventos. En Linux se fuerza el backend X11 (`default-features = false, features = ["x11"]`); en Windows y macOS usa el backend nativo. |
| [`rayon`](https://crates.io/crates/rayon) | 1.12.0 | Paralelismo de datos en el render (reparte las filas entre todos los núcleos). |
| [`num-complex`](https://crates.io/crates/num-complex) | 0.4.6 | Tipo `Complex<f64>` para la aritmética del plano complejo, tanto de `c` (Mandelbrot) como de `z` (Julia). |
| [`image`](https://crates.io/crates/image) | 0.25.10 | Codificación PNG para las capturas (`default-features = false, features = ["png"]`). |
| [`font8x8`](https://crates.io/crates/font8x8) | 0.3.1 | Tipografía bitmap 8×8 para la leyenda en pantalla. |
| [`dirs`](https://crates.io/crates/dirs) | 5.0.1 | Resolución multiplataforma de la carpeta de imágenes del usuario para las capturas. |

## Rendimiento

Referencias aproximadas en un CPU moderno de 8 núcleos, buffer de 1000 × 1000
(válido para ambos modos, ya que el coste por píxel es el mismo):

| Iteraciones | Tiempo por render |
|---|---|
| 1.000 | ~80–150 ms |
| 5.000 | ~300–600 ms |
| 10.000 | ~0,6–1,2 s |
| 50.000 | ~3–6 s |

El uso de Rayon reparte el coste entre todos los núcleos disponibles. El paso
Mandelbrot → Julia (`J`) implica un render completo porque cambia el modo, pero
el coste es idéntico al de cualquier otro re-render.

## Limitaciones conocidas

- **Wayland nativo no soportado en Linux.** La app fuerza el backend X11 de
  `minifb`, que funciona perfectamente a través de Xwayland. Como consecuencia,
  no arranca en compositores Wayland sin Xwayland activado (poco habitual en
  escritorios de usuario).
- **Aviso inofensivo al cerrar (Linux/Wayland).** Al salir pueden aparecer
  mensajes `queue 0x... destroyed while proxies still attached` en la terminal.
  Son emitidos por la capa de Wayland/Xwayland al liberar recursos y no indican
  ningún error real.
- **Precisión limitada a `f64`.** Permite zooms hasta un factor de ~10¹³–10¹⁵
  según la zona explorada. Más allá aparecen imágenes cada vez más pixeladas.
  Este límite aplica tanto a Mandelbrot como a Julia.
- **Sin posicionamiento programático de la ventana.** `minifb` no expone API
  multiplataforma para esto. Si la ventana aparece mal colocada, muévela
  manualmente con el gestor de ventanas.
- **Cambiar de Julia requiere volver a Mandelbrot.** La tecla `J` solo actúa en
  modo Mandelbrot para evitar la ambigüedad de "¿qué `c` tomo?". Si quieres otro
  Julia, pulsa `U` (o `M`) y elige otro punto.

## Reconocimientos

Las paletas Viridis, Inferno, Plasma y Turbo provienen del proyecto Matplotlib,
a su vez basadas en trabajo de Nathaniel Smith, Stéfan van der Walt, Bastian
Bechtold, y otros.

Hot, Flag y Jet son herencia de MATLAB / IDL.

El conjunto de Julia y su relación con el de Mandelbrot (conexidad de `J_c` en
función de la órbita crítica de `c`) son resultados clásicos de la dinámica
compleja; véase por ejemplo *Complex Dynamics* de Carleson y Gamelin, o los
trabajos de Douady y Hubbard.

## Licencia

Este proyecto se distribuye bajo la licencia **MIT**. Consulta el archivo
[LICENSE](LICENSE) para el texto completo.

En resumen: puedes usar, copiar, modificar, fusionar, publicar, distribuir,
sublicenciar y vender copias del software, siempre que preserves el aviso de
copyright original y no se responsabilice al autor de posibles daños.
