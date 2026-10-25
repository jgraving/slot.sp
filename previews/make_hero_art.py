"""Build the site's real wordmark font and a restrained studio-light overlay."""
from pathlib import Path
import io, math, sys, urllib.request
from PIL import Image, ImageDraw, ImageFont
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont
out = Path(sys.argv[1]); out.mkdir(parents=True, exist_ok=True)
font = TTFont(io.BytesIO(urllib.request.urlopen('https://jakegraving.com/assets/fonts/google-sans-flex-v5-web.woff2', timeout=60).read()))
font = instantiateVariableFont(font, {'wght': 600}, inplace=False)
font.flavor = None
buf = io.BytesIO(); font.save(buf); buf.seek(0)
typeface = ImageFont.truetype(buf, 150)
logo = Image.new('RGBA', (1440, 192))
ImageDraw.Draw(logo).text((720, 92), 'slot.sp', font=typeface, fill=(246, 247, 242, 255), anchor='mm', stroke_width=0)
logo.resize((720, 96), Image.Resampling.LANCZOS).save(out/'hero-wordmark.png')
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
