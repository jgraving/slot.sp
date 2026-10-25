"""Bake Google Sans Flex variable axes into a compact, deterministic glass-logo atlas.
Requires Pillow, numpy, scipy, FontTools and Brotli. No font is shipped at runtime.
"""
from pathlib import Path
import argparse, io, math, urllib.request
import numpy as np
from scipy.ndimage import distance_transform_edt, gaussian_filter, shift
from PIL import Image, ImageDraw, ImageFont
from fontTools.ttLib import TTFont

W,H,N,COLS=480,128,48,4

def build(font_path, output):
    if font_path:
        data=Path(font_path).read_bytes()
    else:
        data=urllib.request.urlopen('https://jakegraving.com/assets/fonts/google-sans-flex-v5-web.woff2',timeout=60).read()
    font=TTFont(io.BytesIO(data));font.flavor=None
    buf=io.BytesIO();font.save(buf)
    axes=[a.axisTag for a in font['fvar'].axes]
    atlas=Image.new('RGBA',(W*COLS,H*(N//COLS)))
    yy,xx=np.mgrid[:H,:W].astype(np.float32)
    for frame in range(N):
        t=frame/(N-1)
        # Damped elastic approach, then an exact terminal pose and material.
        q=min(t/.83,1.)
        residual=math.exp(-7*q)*math.cos(10*q) if q<1 else 0.
        loc={'opsz':18+54*max(0,residual),'wdth':100+8*residual,
             'wght':600-250*residual,'ROND':60*max(0,residual),'slnt':-6*max(0,residual)}
        loc={a.axisTag:max(a.minValue,min(a.maxValue,loc[a.axisTag])) for a in font['fvar'].axes}
        face=ImageFont.truetype(io.BytesIO(buf.getvalue()),208)
        face.set_variation_by_axes([loc[k] for k in axes])
        mask=Image.new('L',(W*2,H*2))
        draw=ImageDraw.Draw(mask);text='slot.sp'
        length=draw.textlength(text,font=face);x=W-length/2
        # Per-letter elastic displacement, keeping the final baseline absolutely still.
        for i,ch in enumerate(text):
            advance=draw.textlength(text[:i],font=face)
            offset=22*residual*math.sin(i*1.15+q*4)
            draw.text((x+advance,H+52+offset),ch,font=face,fill=255,anchor='ls')
        mask=mask.resize((W,H),Image.Resampling.LANCZOS)
        m=np.asarray(mask,dtype=np.float32)/255
        dist=distance_transform_edt(m>.5)
        bevel=np.minimum(dist/3.4,1)
        height=np.sin(bevel*np.pi/2)
        gy,gx=np.gradient(gaussian_filter(height,.7))
        nz=1/np.sqrt(1+(gx*4)**2+(gy*4)**2)
        nx=-gx*4*nz;ny=-gy*4*nz
        light=np.clip(-.45*nx-.55*ny+.7*nz,0,1)
        rim=np.exp(-dist/1.2)*m
        sweep=np.exp(-((xx/W-(t*1.7-.25)-yy/H*.16)/.055)**2)
        broad=np.exp(-((xx/W-(t*1.2)+yy/H*.2)/.3)**2)
        mix=np.clip(xx/W*.7+yy/H*.3,0,1)[...,None]
        tint=np.array([.12,.63,.68])*(1-mix)+np.array([.50,.25,.79])*mix
        glass=tint*(.24+.50*light[...,None]) + rim[...,None]*np.array([.30,.55,.65])
        glass+=sweep[...,None]*(.18+.55*rim[...,None])+broad[...,None]*.10
        # The prismatic material resolves to the exact warm-white brand mark.
        settle=np.clip((t-.64)/.22,0,1);settle=settle*settle*(3-2*settle)
        rgb=glass*(1-settle)+np.array([246,247,242])/255*settle
        alpha=m*np.clip(t/.07,0,1)
        # Thin displaced rear faces provide depth before settling flush.
        back=shift(m,(3*(1-settle),3*(1-settle)),order=1,mode='constant')*(1-alpha)*.5*(1-settle)
        rgb=np.clip(rgb,0,1)
        a=np.clip(alpha+back,0,1)
        rgb=(rgb*alpha[...,None]+np.array([.18,.28,.42])*back[...,None])/np.maximum(a[...,None],.001)
        rgba=np.dstack((np.clip(rgb*255,0,255).astype('uint8'),np.round(a*255).astype('uint8')))
        image=Image.fromarray(rgba,'RGBA')
        atlas.paste(image,((frame%COLS)*W,(frame//COLS)*H))
    output=Path(output);output.parent.mkdir(parents=True,exist_ok=True);atlas.save(output,optimize=True)
    print(output,output.stat().st_size)

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('output');p.add_argument('--font');a=p.parse_args();build(a.font,a.output)
