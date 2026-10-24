"""Prepare original label crops and a custom wallpaper for README captures."""
import json
import math
import sys
import shutil
from pathlib import Path
from PIL import Image, ImageDraw, ImageFilter

root = Path(sys.argv[1])
sources = json.loads(Path(__file__).with_name('art-sources.json').read_text())
for name in sources:
    source = Path(__file__).with_name('artwork') / (name.replace(' ', '-') + '.png')
    shutil.copyfile(source, root / 'Labels/GBA' / (name + '.png'))

w, h = 720, 480
im = Image.new('RGBA', (w,h))
pixels = []
for y in range(h):
    for x in range(w):
        blue = math.exp(-((x-110)/340)**2 - ((y-120)/290)**2)
        warm = math.exp(-((x-740)/310)**2 - ((y+40)/310)**2)
        purple = math.exp(-((x-460)/370)**2 - ((y-420)/250)**2)
        pixels.append((round(17+11*blue+68*warm+16*purple),round(23+37*blue+20*warm+5*purple),round(40+50*blue+17*warm+27*purple),255))
im.putdata(pixels)
lines = Image.new('RGBA', (w*2,h*2))
draw = ImageDraw.Draw(lines)
for n in range(22):
    points=[]
    for x in range(-30,760,4):
        y=355+n*18 + 32*math.sin(x/175+n*.11) - x*.29
        points.append((x*2,y*2))
    draw.line(points,fill=(146,171,209,28),width=1)
lines=lines.resize((w,h),Image.Resampling.LANCZOS)
im=Image.alpha_composite(im,lines)
im.save(root/'Wallpapers/twilight.png')
