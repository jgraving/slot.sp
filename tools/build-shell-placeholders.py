#!/usr/bin/env python3
"""Rebuild placeholder lettering from unmodified shell facsimiles.

Pass --source-dir with the original, locally obtained shell assets. Originals
are inputs only; only the cleaned project-marked outputs belong in the repo.
"""
import argparse
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont, ImageFilter

ROOT = Path(__file__).resolve().parents[1]
BANDS = {'gba_neutral.png': (76,286,23,46), 'gba_clear.png': (76,286,23,46),
         'gb_neutral.png': (47,267,41,69), 'gbc_neutral.png': (53,263,39,70),
         'gbc_clear.png': (53,263,39,70)}

def rebuild(source, output, font_path):
    output.mkdir(parents=True, exist_ok=True)
    for name, (left,right,top,bottom) in BANDS.items():
        image = Image.open(source/name).convert('RGBA')
        width = 360 if name.startswith('gba') else 312
        offset = (image.width-width)//2
        pixels = image.load()
        # Follow each column's existing mould shading vertically. Horizontal
        # interpolation flattened the whole logo area into a rectangular badge.
        # Preserve the original edge geometry, grain and alpha outside the mark.
        # Smooth the clean boundary samples across the shoulder so source grain
        # cannot become vertical streaks in the reconstructed surface.
        boundaries=[]
        for y in [top-1,bottom]:
            strip=Image.new('RGB',(right-left,1))
            strip.putdata([pixels[x+offset,y][:3] for x in range(left,right)])
            strip=strip.filter(ImageFilter.GaussianBlur(8))
            boundaries.append(list(strip.getdata()))
        for x in range(left,right):
            a,b = boundaries[0][x-left],boundaries[1][x-left]
            if name.startswith('gba'):
                # The raised old lettering touched the upper sample row.
                a=tuple(max(c,120) for c in a)
            for y in range(top,bottom):
                if name.startswith('gba') and y<27 and max(pixels[x+offset,y][:3])<70:
                    continue
                t,span=y-top+1,bottom-top+1
                pixels[x+offset,y]=tuple((a[c]*(span-t)+b[c]*t+span//2)//span for c in range(3))+(pixels[x+offset,y][3],)
        font = ImageFont.truetype(str(font_path), (bottom-top-4)*4)
        box=font.getbbox('slot.sp')
        mask=Image.new('L',(box[2]-box[0],box[3]-box[1]))
        ImageDraw.Draw(mask).text((-box[0],-box[1]),'slot.sp',font=font,fill=255)
        mask=mask.resize((round(mask.width/4),round(mask.height/4)),Image.Resampling.LANCZOS)
        x0=offset+(width-mask.width)//2
        y0=top+(bottom-top-mask.height)//2
        for y in range(mask.height):
            for x in range(mask.width):
                alpha=mask.getpixel((x,y)); p=pixels[x+x0,y+y0]
                pixels[x+x0,y+y0]=tuple((c*(255-alpha)+max(0,c-40)*alpha+127)//255 for c in p[:3])+(p[3],)
        image.save(output/name)

if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source-dir',type=Path,required=True)
    parser.add_argument('--output-dir',type=Path,default=ROOT/'assets')
    parser.add_argument('--font',type=Path,default=ROOT/'crates/slot-ui/assets/fonts/display.ttf')
    args=parser.parse_args()
    rebuild(args.source_dir,args.output_dir,args.font)
