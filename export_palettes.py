"""
Exporta una paleta de Matplotlib a Rust, dando el nombre y el número de puntos de muestreo.
Necesario instaladar bibliotecas:
-  matplotlib
-  numpy
	
	>pip install matplotlib numpy
"""
import matplotlib
import numpy as np
import sys

def export_rust_palette(name, n=5):
    try:
        cmap = matplotlib.colormaps[name]
    except KeyError:
        print(f"❌ Error: '{name}' no es una paleta válida de Matplotlib.")
        print("   Nombres disponibles (algunos):", ", ".join(sorted(matplotlib.colormaps)[:10]), "...")
        return

    points = np.linspace(0, 1, n)
    rgb = (cmap(points)[:, :3] * 255).astype(int)

    # Nombre de constante válido en Rust: mayúsculas, guiones y puntos -> guión bajo
    const_name = name.upper().replace('-', '_').replace('.', '_')

    print(f"\n// Paleta {name}: {n} puntos de muestreo")
    print(f"const {const_name}: [(f64, (u8, u8, u8)); {n}] = [")
    for t, (r, g, b) in zip(points, rgb):
        print(f"    ({t:.2f}, ({r:3d}, {g:3d}, {b:3d})),")
    print("];\n")


def main():
    print("=== Exportador de paletas Matplotlib → Rust ===\n")

    while True:
        name = input("Nombre de la paleta (Enter para salir): ").strip()
        if not name:
            print("Saliendo.")
            break

        n_str = input(f"  Número de puntos para '{name}' [5]: ").strip()
        n = int(n_str) if n_str else 5

        if n < 2:
            print("  ⚠️  Se necesitan al menos 2 puntos. Usando 5.")
            n = 5

        export_rust_palette(name, n)

        otro = input("¿Otra paleta? [s/N]: ").strip().lower()
        if otro != 's':
            print("Saliendo.")
            break


if __name__ == "__main__":
    main()