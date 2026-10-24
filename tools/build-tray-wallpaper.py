"""Build the default twilight wallpaper and its blurred menu companion."""
from pathlib import Path
import math
import random
from PIL import Image, ImageFilter

root = Path(__file__).resolve().parents[1]
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
im.save(root/'assets/background_default.png')
im.filter(ImageFilter.GaussianBlur(6)).save(root/'assets/background_default_blur.png')
