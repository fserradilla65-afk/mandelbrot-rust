#!/usr/bin/env python3
"""
Monta una secuencia de frames PNG en un vídeo MP4.

Uso:
    python mount.py                    # usa la carpeta frames_* más reciente
    python mount.py --fps 60           # cambia los FPS
    python mount.py -o MandelMandelzoom.mp4        # cambia el nombre de salida
    python mount.py -d Saves/frames_1730000000   # elige una carpeta concreta
"""

import argparse
import re
import sys
from pathlib import Path

import imageio.v2 as imageio


def find_latest_frames_dir(saves_dir: Path) -> Path:
    candidates = sorted(
        (d for d in saves_dir.iterdir()
         if d.is_dir() and d.name.startswith("frames_")),
        key=lambda d: d.name,
        reverse=True,
    )
    if not candidates:
        sys.exit(f"No se encontró ninguna carpeta frames_* en {saves_dir}")
    return candidates[0]


def frame_number(path: Path) -> int:
    m = re.search(r"frame_(\d+)", path.name)
    return int(m.group(1)) if m else -1


def main() -> None:
    ap = argparse.ArgumentParser(
        description="Monta una secuencia de frames PNG en un vídeo MP4.",
    )
    ap.add_argument("-d", "--dir",
                    help="Carpeta de frames (por defecto: la más reciente)")
    ap.add_argument("-o", "--output", default="Mandelzoom.mp4",
                    help="Nombre del vídeo de salida (por defecto: Mandelzoom.mp4)")
    ap.add_argument("--fps", type=int, default=30,
                    help="Frames por segundo (por defecto: 30)")
    args = ap.parse_args()

    script_dir = Path(__file__).resolve().parent
    saves_dir = script_dir / "Saves"
    if not saves_dir.is_dir():
        sys.exit(f"No existe la carpeta {saves_dir}")

    if args.dir:
        frames_dir = Path(args.dir)
        if not frames_dir.is_absolute():
            frames_dir = script_dir / frames_dir
    else:
        frames_dir = find_latest_frames_dir(saves_dir)

    if not frames_dir.is_dir():
        sys.exit(f"No existe la carpeta {frames_dir}")

    frames = sorted(frames_dir.glob("frame_*.png"), key=frame_number)
    if not frames:
        sys.exit(f"No hay frames en {frames_dir}")

    output = Path(args.output)
    if not output.is_absolute():
        output = saves_dir / output

    print(f"Carpeta:  {frames_dir.name}")
    print(f"Frames:   {len(frames)}")
    print(f"FPS:      {args.fps}")
    print(f"Duración: {len(frames) / args.fps:.2f} s")
    print(f"Salida:   Saves/{output.name}")

    with imageio.get_writer(output, fps=args.fps, codec="libx264",
                            quality=8, pixelformat="yuv420p") as writer:
        for i, f in enumerate(frames, 1):
            writer.append_data(imageio.imread(f))
            if i % 30 == 0 or i == len(frames):
                print(f"  {i}/{len(frames)}", end="\r", flush=True)
    print()
    print("Listo.")


if __name__ == "__main__":
    main()
