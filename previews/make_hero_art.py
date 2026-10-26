"""Build the fixed lighting overlay for the production hero capture."""
from pathlib import Path
import math, sys
from PIL import Image
out = Path(sys.argv[1]); out.mkdir(parents=True, exist_ok=True)
light = Image.new('RGBA', (720, 480))
pixels = []
for y in range(480):
    for x in range(720):
        warm = math.exp(-((x-180)/210)**2 - ((y-170)/260)**2)
        cool = math.exp(-((x-670)/230)**2 - ((y-260)/300)**2)
        weight = warm + cool*.55
        alpha = round(8*min(1,weight))
        if weight:
            color = tuple(round((a*warm+b*cool*.55)/weight) for a,b in zip((255,239,211),(160,145,210)))
        else: color = (0,0,0)
        pixels.append(color+(alpha,))
light.putdata(pixels); light.save(out/'hero-light.png')
