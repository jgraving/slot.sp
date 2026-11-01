#!/usr/bin/env python3
"""Extract the exact firmware wordmark with smooth transparency, without retyping it."""
from pathlib import Path
import argparse
from PIL import Image

root = Path(__file__).resolve().parents[1]
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--output', type=Path, default=root / 'site/docs/assets/wordmark.png')
args = p.parse_args()
image = Image.open(root / 'card/System/bootlogo.bmp').convert('RGB')
background, ink = image.getpixel((0, 0)), (246, 247, 242)
delta = [f - b for f, b in zip(ink, background)]
norm = sum(d * d for d in delta)
alpha = Image.new('L', image.size)
for y in range(image.height):
    for x in range(image.width):
        pixel = image.getpixel((x, y))
        coverage = sum((c - b) * d for c, b, d in zip(pixel, background, delta)) / norm
        alpha.putpixel((x, y), round(255 * max(0, min(1, coverage))))
result = Image.new('RGBA', image.size, ink + (255,))
result.putalpha(alpha)
result = result.crop(result.getbbox())
args.output.parent.mkdir(parents=True, exist_ok=True)
result.save(args.output)
print(f'Extracted firmware wordmark: {result.width}×{result.height}')
