#!/usr/bin/env python3
"""Apply the lowercase project wordmark to bundled shell logo bands."""
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont
ROOT = Path(__file__).resolve().parents[1]
BANDS = {'gba_neutral.png': (76,286,23,46), 'gba_clear.png': (76,286,23,46),
         'gb_neutral.png': (47,267,41,69), 'gbc_neutral.png': (53,263,39,70),
         'gbc_clear.png': (53,263,39,70)}
for name, (left,right,top,bottom) in BANDS.items():
    path = ROOT/'assets'/name
    image = Image.open(path).convert('RGBA')
    # Wider source assets are centre cropped by the renderer.
    width = 360 if name.startswith('gba') else 312
    offset = (image.width-width)//2
    pixels = image.load()
    for y in range(top,bottom):
        a,b = pixels[left-1+offset,y],pixels[right+offset,y]
        for x in range(left,right):
            if name.startswith("gba") and y < 27 and max(pixels[x+offset,y][:3]) < 70: continue
            t,span=x-left+1,right-left+1
            pixels[x+offset,y]=tuple((a[c]*(span-t)+b[c]*t+span//2)//span for c in range(3))+(pixels[x+offset,y][3],)
    font = ImageFont.truetype(str(ROOT/'crates/slot-ui/assets/fonts/display.ttf'), (bottom-top-4)*4)
    box=font.getbbox('slot.sp'); mask=Image.new('L',(box[2]-box[0],box[3]-box[1]))
    ImageDraw.Draw(mask).text((-box[0],-box[1]),'slot.sp',font=font,fill=255)
    mask=mask.resize((round(mask.width/4),round(mask.height/4)),Image.Resampling.LANCZOS)
    x0=offset+(width-mask.width)//2; y0=top+(bottom-top-mask.height)//2
    for y in range(mask.height):
        for x in range(mask.width):
            alpha=mask.getpixel((x,y)); p=pixels[x+x0,y+y0]
            pixels[x+x0,y+y0]=tuple((c*(255-alpha)+max(0,c-40)*alpha+127)//255 for c in p[:3])+(p[3],)
    image.save(path)
