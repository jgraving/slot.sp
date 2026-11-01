#!/usr/bin/env python3
"""Build the website/README wordmark from the current UI's rounded display font."""
import argparse
import hashlib
import json
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

root = Path(__file__).resolve().parents[1]
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--output', type=Path, default=root / 'site/docs/assets/wordmark.png')
args = p.parse_args()
font_path = root / 'crates/slot-ui/assets/fonts/display.ttf'
profile = json.loads((font_path.parent / 'profiles.json').read_text())['profiles']['display']
assert hashlib.sha256(font_path.read_bytes()).hexdigest() == profile['sha256']
font = ImageFont.truetype(str(font_path), 360)
left, top, right, bottom = font.getbbox('slot.sp')
image = Image.new('RGBA', (right-left+8, bottom-top+8))
ImageDraw.Draw(image).text((4-left, 4-top), 'slot.sp', font=font, fill=(246,247,242,255))
args.output.parent.mkdir(parents=True, exist_ok=True)
image.save(args.output)
print(f'Built rounded Google Sans Flex wordmark: {image.width}×{image.height}')
