#!/usr/bin/env python3
"""Tile snapshots into 3x3 contact sheets: grid.py <out-prefix> <png>..."""
import sys
from PIL import Image, ImageDraw

prefix, paths = sys.argv[1], sys.argv[2:]
for sheet, start in enumerate(range(0, len(paths), 9)):
    chunk = paths[start:start + 9]
    tiles = [Image.open(p).convert("RGB") for p in chunk]
    w, h = tiles[0].width // 3, tiles[0].height // 3
    out = Image.new("RGB", (w * 3, h * 3), "white")
    for i, (tile, path) in enumerate(zip(tiles, chunk)):
        tile = tile.resize((w, h), Image.LANCZOS)
        ImageDraw.Draw(tile).rectangle([0, 0, w - 1, h - 1], outline="red")
        out.paste(tile, ((i % 3) * w, (i // 3) * h))
    out.save(f"{prefix}{sheet}.png")
    print(f"{prefix}{sheet}.png")
