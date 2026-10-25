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
        # Travel through the actual extremes of font space before a short
        # damped lock-in: fine/wide -> dense/narrow -> soft/round -> final.
        knots=[0.,.24,.48,.72,.88,1.]
        poses=[(72,108,350,60,-6),(10,95,700,0,0),(64,108,380,60,-6),
               (28,96,690,18,0),(18,100,600,0,0),(18,100,600,0,0)]
        segment=min(np.searchsorted(knots,t,side='right')-1,4)
        u=(t-knots[segment])/(knots[segment+1]-knots[segment])
        u=u*u*(3-2*u)
        values=[a+(b-a)*u for a,b in zip(poses[segment],poses[segment+1])]
        loc=dict(zip(['opsz','wdth','wght','ROND','slnt'],values))
        q=min(t/.88,1.)
        residual=(1-q)**2 * math.cos(q*9) if q<1 else 0.
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
        rim=np.exp(-np.maximum(dist-1.0,0)/1.2)*m
        sweep=np.exp(-((xx/W-(t*1.7-.25)-yy/H*.16)/.055)**2)
        broad=np.exp(-((xx/W-(t*1.2)+yy/H*.2)/.3)**2)
        mix=np.clip(xx/W*.7+yy/H*.3,0,1)[...,None]
        tint=np.array([.12,.63,.68])*(1-mix)+np.array([.50,.25,.79])*mix
        # Glass transmits the scene; the bevels and moving softboxes carry color.
        inner=np.exp(-((dist-3.0)/.9)**2)*m
        glass=tint*(.12+.36*light[...,None]) + rim[...,None]*np.array([.38,.65,.76])
        glass+=inner[...,None]*tint*.45
        glass+=sweep[...,None]*(.28+.65*rim[...,None])+broad[...,None]*.15
        # The prismatic material resolves to the exact warm-white brand mark.
        settle=np.clip((t-.79)/.09,0,1);settle=settle*settle*(3-2*settle)
        rgb=glass*(1-settle)+np.array([246,247,242])/255*settle
        transmission=.12+.62*rim+.20*inner+.28*sweep*m
        alpha=m*np.clip(t/.06,0,1)*((1-settle)*np.clip(transmission,0,1)+settle)
        # Thin displaced rear faces provide depth before settling flush.
        back=shift(m,(3*(1-settle),3*(1-settle)),order=1,mode='constant')*(1-alpha)*.20*(1-settle)*np.clip(t/.06,0,1)
        rgb=np.clip(rgb,0,1)
        a=np.clip(alpha+back,0,1)
        rgb=(rgb*alpha[...,None]+np.array([.18,.28,.42])*back[...,None])/np.maximum(a[...,None],.001)
        rgba=np.dstack((np.clip(rgb*255,0,255).astype('uint8'),np.round(a*255).astype('uint8')))
        if frame == 0: rgba[:] = 0  # No rear-face ghost at the loop boundary.
        image=Image.fromarray(rgba,'RGBA')
        atlas.paste(image,((frame%COLS)*W,(frame//COLS)*H))
    output=Path(output);output.parent.mkdir(parents=True,exist_ok=True);atlas.save(output,optimize=True)
    print(output,output.stat().st_size)

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('output');p.add_argument('--font');a=p.parse_args();build(a.font,a.output)
