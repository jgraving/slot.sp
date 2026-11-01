#!/usr/bin/env python3
"""Package current production UI captures and accessible, loopable hero clips."""
import argparse,hashlib,json,os,shutil,subprocess
from pathlib import Path
from PIL import Image
p=argparse.ArgumentParser();p.add_argument('root',type=Path);args=p.parse_args();root=args.root
repo=Path(__file__).resolve().parents[1];publish=root/'publish';current=publish/'previews/current';homepage=publish/'previews/homepage'
stills=['carousel','settings','general','screen','sound','parent-settings','parent-code','parent-time','kid-menu','in-game-menu','in-game-settings','charging','boot','power','controls','about','favorites','recents','approved','core-picker','save-states','screenshots','link-host','link-join','buttons-shortcuts','kid-warning']
for theme in ['dark','light']:
 for name in stills:assert (root/'raw/current'/theme/f'{name}.png').is_file(), f'Missing {theme}/{name} capture'
 for name in ['settings','parent-code','charging','boot']:assert (root/'raw/current'/theme/f'{name}.mp4').is_file(), f'Missing {theme}/{name} animation'
 for finish in ['graphite','platinum','pearl_blue','pearl_pink']:
  for name in ['hero-native.mp4','hero-boot.mp4','hero.png']:assert (root/'raw/homepage'/f'{theme}-{finish}'/name).is_file(), f'Missing {theme}/{finish}/{name}'
for directory in [current,homepage]:
 if directory.exists():shutil.rmtree(directory)
 directory.mkdir(parents=True,exist_ok=True)
def run(*args):subprocess.run(list(map(str,args)),check=True)
def copy_media(source,target):
 if source.suffix=='.png':
  with Image.open(source) as image:
   assert image.size==(720,480), source
   image.convert('RGB').save(target,optimize=True,compress_level=9)
 else:shutil.copyfile(source,target)
def duration(path):return float(subprocess.check_output(['ffprobe','-v','error','-show_entries','format=duration','-of','default=noprint_wrappers=1:nokey=1',str(path)],text=True))
for theme in ['dark','light']:
 source=root/'raw/current'/theme;target=current/theme;target.mkdir(exist_ok=True)
 if source.exists():
  for file in source.iterdir():
   if file.suffix in ('.mp4','.png') and not (theme=='light' and file.stem in ['screenshots','link-host','link-join','save-states']):copy_media(file,target/file.name)
 for finish in ['graphite','platinum','pearl_blue','pearl_pink']:
  source=root/'raw/homepage'/f'{theme}-{finish}'
  if not source.exists():continue
  a=source/'hero-native.mp4';b=source/'hero-boot.mp4';target=homepage/f'hero-{theme}-{finish}.mp4';fade=.3;la=duration(a);lb=duration(b)
  # Join real carousel and production boot, then crossfade back into the initial
  # carousel pause. Every finish and theme shares the exact same 60Hz timeline.
  filters=('[0:v]format=yuv444p,settb=1/60,setpts=PTS-STARTPTS,split=2[a][c];'
           '[1:v]format=yuv444p,settb=1/60,setpts=PTS-STARTPTS[b];'
           f'[a][b]xfade=transition=fade:duration={fade}:offset={la-fade}[ab];'
           f'[ab][c]xfade=transition=fade:duration={fade}:offset={la+lb-2*fade},'
           f'trim=start={fade}:end={la+lb-fade},setpts=PTS-STARTPTS[v]')
  run('ffmpeg','-nostdin','-v','error','-y','-filter_complex_threads','1','-i',a,'-i',b,'-filter_complex',filters,'-map','[v]','-an','-c:v','libx264','-threads','2','-preset','slow','-crf','23','-pix_fmt','yuv420p','-movflags','+faststart',target)
  copy_media(source/'hero.png',homepage/f'hero-{theme}-{finish}.png')
  if finish=='platinum' and (current/theme).exists():shutil.copyfile(a,current/theme/'carousel.mp4')
# Compact opaque GIF fallbacks for GitHub, never the website's primary media.
for name in ['boot','parent-code','charging']:
 source=current/'light'/f'{name}.mp4';frames=root/f'gif-{name}';frames.mkdir(exist_ok=True)
 run('ffmpeg','-nostdin','-v','error','-y','-i',source,'-vf','fps=12.5',frames/'%04d.png')
 run('python3',repo/'tools/opaque_preview_gif.py',frames,current/'light'/f'{name}.gif','--width','480','--frame-ms','80')
files=[]
for f in sorted(publish.rglob('*')):
 if f.is_file() and f.name!='provenance.json':files.append(dict(path=str(f.relative_to(publish)),bytes=f.stat().st_size,sha256=hashlib.sha256(f.read_bytes()).hexdigest()))
capture_source=os.environ.get('GITHUB_SHA') or subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()
source=os.environ.get('UI_RELEASE_REV') or capture_source
manifest=dict(source_commit=source,capture_source_commit=capture_source,ui_release_commit=os.environ.get('UI_RELEASE_REV','4b3c6f2bdf387e74b3ffd67ff1f81630318c853e'),run_url=(f'https://github.com/{os.environ["GITHUB_REPOSITORY"]}/actions/runs/{os.environ["GITHUB_RUN_ID"]}' if os.environ.get('GITHUB_RUN_ID') else None),capture='previews/site_refresh.rs',rendering='Production Frontend and native desktop GL compositor; deterministic software fixtures with repository sample label artwork, synthetic game headers and mock-core gameplay; not photographs or hardware test results.',dimensions=[720,480],fps=60,files=files)
(current/'provenance.json').write_text(json.dumps(manifest,indent=2)+'\n')
(homepage/'provenance.json').write_text(json.dumps({**manifest,'files':[f for f in files if f['path'].startswith('previews/homepage/')]},indent=2)+'\n')
size=sum(f['bytes'] for f in files)
assert size<25*1024*1024,f'Media budget exceeded: {size/1024/1024:.1f}MiB'
print(f'Collected {len(files)} files ({size/1024/1024:.1f} MiB) from {source}')
