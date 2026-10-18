#!/usr/bin/env python3
"""Tile snapshot PNGs into a 2x2 contact sheet at half resolution.

Usage: uv run --project scripts scripts/montage.py out.png a.png b.png [c.png d.png]
"""
import sys
from PIL import Image

out, paths = sys.argv[1], sys.argv[2:]
images = [Image.open(p).convert("RGB") for p in paths]
w, h = images[0].size
hw, hh = w // 2, h // 2
cols = 2 if len(images) > 1 else 1
rows = (len(images) + cols - 1) // cols
sheet = Image.new("RGB", (hw * cols, hh * rows), "white")
for i, im in enumerate(images):
    sheet.paste(im.resize((hw, hh), Image.LANCZOS), ((i % cols) * hw, (i // cols) * hh))
sheet.save(out)
