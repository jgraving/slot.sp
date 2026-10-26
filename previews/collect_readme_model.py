"""Collect the illustrated README hero, its reduced-motion poster and provenance."""
import hashlib
import json
import os
from pathlib import Path
import sys
from PIL import Image
root = Path(sys.argv[1])
target = root / 'publish/previews/readme-model'
target.mkdir(parents=True, exist_ok=True)
gif = target / 'hero.gif'
with Image.open(gif) as animation:
    size = animation.size
    assert animation.n_frames > 120
    assert 'transparency' not in animation.info
    assert gif.stat().st_size < 20_000_000, 'README hero should remain reasonably sized'
with Image.open(root / 'model-frames/0000.png') as first:
    first.convert('RGB').resize(size, Image.Resampling.LANCZOS).save(target / 'hero.png')
meta = json.loads((root / 'model-capture.json').read_text())
meta.update(source_commit=os.environ['GITHUB_SHA'],
            run_url=f'https://github.com/{os.environ["GITHUB_REPOSITORY"]}/actions/runs/{os.environ["GITHUB_RUN_ID"]}',
            files=[dict(path=f'previews/readme-model/{p.name}', bytes=p.stat().st_size,
                        sha256=hashlib.sha256(p.read_bytes()).hexdigest()) for p in [gif, target / 'hero.png']])
(target / 'provenance.json').write_text(json.dumps(meta, indent=2) + '\n')
print('Collected illustrated README hero and matching poster:', size)
