#!/usr/bin/env python3
"""Render the project's original geometric demo-label collection.

These are documentation designs, not games. No network resources are used.
The editable drawing instructions and resulting PNGs share the project license.
"""
from pathlib import Path
import math
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
DESIGNS = {
    'Fern': ('#143e35', '#b8de98', 'A LITTLE ROOM TO GROW'),
    'Orbit': ('#242b56', '#ffc98d', 'TAKE THE LONG WAY HOME'),
    'Rosette': ('#692f37', '#ffbd85', 'KEEP A LITTLE LIGHT'),
    'Tide': ('#17485b', '#a7e2df', 'FOLLOW YOUR OWN CURRENT'),
    'Wander': ('#75507b', '#ffe0b1', 'SOMETHING NEW AWAITS'),
    'Solstice': ('#65502a', '#f5dc9d', 'A MOMENT IN THE SUN'),
    'Meadow': ('#3e5352', '#d3dbb0', 'MAKE SPACE TO WANDER'),
}

def build(target=ROOT / 'previews/artwork'):
    target.mkdir(parents=True, exist_ok=True)
    font = ROOT / 'crates/slot-ui/assets/fonts/display.ttf'
    title_font = ImageFont.truetype(str(font), 112)
    small_font = ImageFont.truetype(str(font), 23)
    for number, (name, (ground, accent, caption)) in enumerate(DESIGNS.items(), 1):
        image = Image.new('RGB', (1280, 661), ground)
        draw = ImageDraw.Draw(image)
        # A distinct abstract landscape for each entry, built from simple shapes.
        if name == 'Fern':
            draw.line([(950, 520), (950, 120)], fill=accent, width=8)
            for y in range(170, 480, 65):
                draw.ellipse((840, y-55, 950, y+10), fill=accent)
                draw.ellipse((950, y-85, 1060, y-20), fill=accent)
        elif name in ('Orbit', 'Solstice'):
            draw.ellipse((820, 180, 1100, 460), fill=accent)
            for offset in (0, 32, 64):
                draw.ellipse((720-offset, 255-offset//2, 1200+offset, 385+offset//2), outline=accent, width=4)
            draw.ellipse((1140, 190, 1160, 210), fill=accent)
        elif name in ('Rosette', 'Wander'):
            draw.ellipse((830, 130, 1110, 410), fill=accent)
            for i in range(5):
                y = 340 + i*38
                draw.rounded_rectangle((760-i*12, y, 1180+i*12, y+16), radius=8, fill=ground)
        else:
            for line in range(7):
                points = [(x, 190+line*42+int(28*math.sin(x/85+number))) for x in range(770, 1210, 3)]
                draw.line(points, fill=accent, width=7)
        draw.text((66, 53), f'SLOT.SP  /  STUDY {number:02}', font=small_font, fill=accent)
        draw.text((60, 220), name, font=title_font, fill=accent)
        draw.text((66, 388), caption, font=small_font, fill=accent)
        draw.line((66, 540, 1214, 540), fill=accent, width=2)
        draw.text((66, 575), 'ORIGINAL DEMO COLLECTION', font=small_font, fill=accent)
        image.save(target / f'{name}.png', optimize=True)

if __name__ == '__main__':
    build()
