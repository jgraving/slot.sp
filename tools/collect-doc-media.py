#!/usr/bin/env python3
"""Collect native documentation captures; optionally prepare GitHub blobs for review.

This never moves a branch or commits. The caller reviews the artifact and uses
its GitHub integration to commit the listed blob SHAs.
"""
import argparse,base64,hashlib,json,os,shutil,subprocess,urllib.request
from pathlib import Path
from PIL import Image
p=argparse.ArgumentParser();p.add_argument('root',type=Path);p.add_argument('--git-objects',action='store_true');args=p.parse_args()
root=args.root;out=root/'out';publish=root/'publish';repo=Path(__file__).resolve().parents[1]
if not args.git_objects:
    target=publish/'previews';shots=target/'guide';shots.mkdir(parents=True,exist_ok=True)
    videos={'carousel':'readme-carousel','core-picker':'readme-core-picker','link':'readme-link','shutdown':'readme-shutdown','favorite-remove':'favorite-remove','hero-glass':'hero-glass','boot-brand':'boot-brand','cart-motion':'cart-motion','lava':'lava-gradient'}
    for name,stem in videos.items():
        source=out/(name+'.mp4');shutil.copyfile(source,target/(stem+'.mp4'))
        frames=root/('frames-'+name);frames.mkdir(exist_ok=True)
        subprocess.run(['ffmpeg','-nostdin','-v','error','-y','-i',str(source),'-vf','fps=20',str(frames/'%04d.png')],check=True)
        gif='slot-fit-motion' if name=='cart-motion' else stem
        subprocess.run(['python3',str(repo/'tools/opaque_preview_gif.py'),str(frames),str(target/(gif+'.gif')),'--width','480'],check=True)
        with Image.open(next(iter(sorted(frames.glob('*.png'))))) as first:
            first.convert('RGB').save(target/(stem+'.png'))
    shutil.copyfile(out/'hero-glass.png',target/'hero-glass.png')
    # Native selected shots are clearer posters than intermediate animation frames.
    for name in ['carousel','core-picker','favorites','approved']:
        shutil.copyfile(out/(name+'.png'),shots/(name+'.png'))
    for name in ['parent-settings','parent-time','kid-menu','kid-warning','parent-code','sound','about']:
        shutil.copyfile(out/(name+'.png'),shots/(name+'.png'))
    for scene,name in [('menu-general','appearance'),('menu-general-bottom','general'),('menu-power','power')]:
        if scene=='menu-power':
            # The compositor script's menu list is fixed; export this scene by the same routine.
            # Copy into a temporary standard scene name, keeping the production draw list.
            temp=root/'power-layout';shutil.copytree(root/'layout'/scene,temp/'menu-general',dirs_exist_ok=True)
            for other in ['menu-general-bottom','menu-game-top','menu-game-bottom']:
                shutil.copytree(root/'layout'/scene,temp/other,dirs_exist_ok=True)
            subprocess.run(['python3',str(repo/'tools/compose-layout-review.py'),str(temp),str(out/'power-overview.png'),'menu'],check=True)
            source=temp/'menu-general/frame.png'
        else: source=root/'layout'/scene/'frame.png'
        shutil.copyfile(source,shots/(name+'.png'))
    shutil.copyfile(target/'readme-shutdown.gif',target/'power-switch.gif')
    subprocess.run(['python3',str(repo/'tools/opaque_preview_gif.py'),str(out/'pin-frames'),str(target/'parent-code.gif'),'--width','480'],check=True)
    subprocess.run(['ffmpeg','-nostdin','-v','error','-y','-framerate','20','-i',str(out/'pin-frames/%04d.png'),'-c:v','libx264','-pix_fmt','yuv420p','-movflags','+faststart',str(target/'parent-code.mp4')],check=True)
    shutil.copyfile(out/'parent-code.png',target/'parent-code.png')
    files=[]
    for f in sorted(publish.rglob('*')):
        if f.is_file():files.append(dict(path=str(f.relative_to(publish)),bytes=f.stat().st_size,sha256=hashlib.sha256(f.read_bytes()).hexdigest()))
    (target/'media-provenance.json').write_text(json.dumps(dict(source_commit=os.environ.get('GITHUB_SHA'),run_url=f'https://github.com/{os.environ.get("GITHUB_REPOSITORY")}/actions/runs/{os.environ.get("GITHUB_RUN_ID")}',files=files),indent=2)+'\n')
    print('Collected',len(files),'native captures')
else:
    entries=[];token=os.environ['GH_TOKEN'];api='https://api.github.com/repos/'+os.environ['GITHUB_REPOSITORY']+'/git/blobs'
    for file in sorted(publish.rglob('*')):
        if not file.is_file():continue
        data=json.dumps(dict(content=base64.b64encode(file.read_bytes()).decode(),encoding='base64')).encode()
        request=urllib.request.Request(api,data=data,headers={'Authorization':'Bearer '+token,'Accept':'application/vnd.github+json','Content-Type':'application/json'},method='POST')
        with urllib.request.urlopen(request,timeout=120) as response:blob=json.load(response)
        entries.append(dict(path=str(file.relative_to(publish)),mode='100644',type='blob',sha=blob['sha']))
        print(file.relative_to(publish),blob['sha'])
    (root/'media-entries.json').write_text(json.dumps(entries,indent=2)+'\n')
