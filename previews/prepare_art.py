"""Prepare original label crops and a custom wallpaper for README captures."""
import json
import math
import sys
import shutil
import random
from pathlib import Path
from PIL import Image

root = Path(sys.argv[1])
sources = json.loads(Path(__file__).with_name('art-sources.json').read_text())
for name in sources:
    source = Path(__file__).with_name('artwork') / (name.replace(' ', '-') + '.png')
    shutil.copyfile(source, root / 'Labels/GBA' / (name + '.png'))

w, h = 720, 480
im = Image.new('RGBA', (w, h))
pixels = []
rng = random.Random(720480)
for y in range(h):
    t = min(1.0, max(0.0, (y - 260) / 170))
    glow = 1.0 - t*t*(3.0 - 2.0*t)
    for x in range(w):
        teal = math.exp(-((x-70)/370)**2 - ((y-80)/300)**2)
        violet = math.exp(-((x-670)/370)**2 - ((y-90)/300)**2)
        rose = math.exp(-((x-500)/330)**2 - ((y+130)/290)**2)
        grain = rng.uniform(-0.65, 0.65)
        rgb = (14 + glow*(4*teal+40*violet+24*rose),
               21 + glow*(34*teal+6*violet+5*rose),
               35 + glow*(48*teal+51*violet+14*rose))
        pixels.append(tuple(round(v + grain) for v in rgb) + (255,))
im.putdata(pixels)
im.save(root/'Wallpapers/twilight.png')
