"""Render the slot.sp optical identity with Blender Cycles, then pack its runtime atlas.

Requires Blender4.5, Pillow and FontTools (+Brotli for the site-hosted WOFF2).
The device only reads the baked PNG; no renderer or font ships at runtime.
"""
from pathlib import Path
import argparse, importlib.util, math, os, subprocess, tempfile
from PIL import Image

ROOT=Path(__file__).resolve().parents[1]

def pack(frames,output):
    atlas=Image.new('RGBA',(1920,1536))
    for i in range(48):
        im=Image.open(Path(frames)/f'{i:04d}.png').convert('RGBA').resize((480,128),Image.Resampling.LANCZOS)
        # Fade the first grazing reflection in. No ghost at the exact loop boundary.
        t=i/47
        if i==0: im=Image.new('RGBA',im.size)
        elif t<.07: im.putalpha(im.getchannel('A').point(lambda a:round(a*t/.07)))
        if t>=.96:
            white=Image.new('RGBA',im.size,(246,247,242,0));white.putalpha(im.getchannel('A'));im=white
        atlas.paste(im,((i%4)*480,(i//4)*128))
    atlas.save(output,optimize=True)
    final=atlas.crop((1440,1408,1920,1536));final=final.crop(final.getbbox())
    bmp=Image.new('RGB',(final.width+32,final.height+32),(11,14,17));bmp.paste(final,(16,16),final)
    bmp.save(ROOT/'card/System/bootlogo.bmp')
    print('atlas',output,Path(output).stat().st_size)

def vignette():
    im=Image.new('RGBA',(720,480));p=[]
    for y in range(480):
        for x in range(720):
            a=round(165*math.exp(-((x-360)/315)**4-((y-165)/170)**4))
            p.append((5,10,18,a))
    im.putdata(p);im.save(ROOT/'assets/brand-vignette.png',optimize=True)

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('output');p.add_argument('--font');p.add_argument('--blender',default='blender');p.add_argument('--frames-dir');p.add_argument('--samples',default=128,type=int);a=p.parse_args()
    vignette()
    if a.frames_dir:pack(a.frames_dir,a.output)
    else:
        with tempfile.TemporaryDirectory(prefix='slot-brand-') as temp:
            spec=importlib.util.spec_from_file_location('geometry',ROOT/'tools/build-brand-geometry.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
            geometry=Path(temp)/'geometry.json';m.generate(geometry,a.font)
            env=os.environ.copy();lib=Path(a.blender).resolve().parent/'lib'
            if lib.is_dir():env['LD_LIBRARY_PATH']=str(lib)+(':'+env['LD_LIBRARY_PATH'] if env.get('LD_LIBRARY_PATH') else '')
            subprocess.run([a.blender,'-b','--factory-startup','--python',str(ROOT/'tools/render-brand-glass.py'),'--',str(geometry),temp,'--width','960','--height','256','--samples',str(a.samples)],check=True,env=env)
            pack(temp,a.output)
