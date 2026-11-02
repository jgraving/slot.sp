"""Collect model-rendered opening clips, posters, README hero and provenance."""
import hashlib, json, os, shutil, sys
from fractions import Fraction
from pathlib import Path
import av
from PIL import Image
from build_model_closing import build_closing
root=Path(sys.argv[1]);publish=root/'publish';target=publish/'previews/readme-model';target.mkdir(parents=True,exist_ok=True)
meta=json.loads((root/'model-capture.json').read_text())
gif=(target/'hero.gif').rename(target/'hero-3d.gif')
with Image.open(gif) as animation:
    size=animation.size
    assert animation.n_frames>40
    duration_ms=0
    for i in range(animation.n_frames):
        animation.seek(i);duration_ms+=animation.info['duration']
    assert duration_ms==meta['frames']*meta['frame_ms']
    assert 'transparency' not in animation.info
    assert gif.stat().st_size<20_000_000
with Image.open(root/'model-frames'/f'{meta["poster_index"]:04}.png') as first:
    first.convert('RGB').resize(size,Image.Resampling.LANCZOS).save(target/'hero-3d.png')
revision=meta['model_sha256'][:12]
model=publish/'previews/sp-model'/revision;model.mkdir(parents=True,exist_ok=True)
for name in ['platinum','graphite','pearl_blue','pearl_pink','gold']:
    for pose in ['open','closed']:shutil.copyfile(root/f'{name}-{pose}.png',model/f'{name}-{pose}.png')
    with av.open(str(model/f'{name}-opening.mp4'),'w',options={'movflags':'+faststart'}) as clip:
        stream=clip.add_stream('libx264',rate=30);stream.width=720;stream.height=900;stream.pix_fmt='yuv420p';stream.options={'crf':'19','preset':'medium'}
        for i,p in enumerate(sorted((root/'opening'/name).glob('*.png'))):
            frame=av.VideoFrame.from_image(Image.open(p).convert('RGB'));frame.pts=i;frame.time_base=Fraction(1,30)
            for packet in stream.encode(frame):clip.mux(packet)
        for packet in stream.encode():clip.mux(packet)
shutil.copyfile(root/'projection.json',model/'projection.json')
derived_closing=build_closing(model,publish)
def entries(directory):
    return [dict(path=str(p.relative_to(publish)),bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest()) for p in sorted(directory.rglob('*')) if p.is_file() and p.name!='provenance.json']
source=dict(source_commit=os.environ['GITHUB_SHA'],run_url=f'https://github.com/{os.environ["GITHUB_REPOSITORY"]}/actions/runs/{os.environ["GITHUB_RUN_ID"]}')
(target/'provenance.json').write_text(json.dumps(dict(meta,**source,files=entries(target)),indent=2)+'\n')
(publish/'previews/sp-model/provenance.json').write_text(json.dumps(dict(**source,revision=revision,model_sha256=meta['model_sha256'],derived_closing=derived_closing,files=entries(model)),indent=2)+'\n')
print('Collected production 3D opening clips, posters and README hero:',revision)
