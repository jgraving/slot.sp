"""Collect four native homepage finishes with their source and media hashes."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import sys

root = Path(sys.argv[1])
target = root / 'publish/previews/homepage'
target.mkdir(parents=True, exist_ok=True)
files = []
for finish in ['graphite', 'platinum', 'pearl_blue', 'pearl_pink']:
    for suffix in ['mp4', 'png']:
        source = root / f'out/{finish}/hero.{suffix}'
        dest = target / f'hero-{finish}.{suffix}'
        shutil.copyfile(source, dest)
        files.append(dict(path=f'previews/homepage/{dest.name}', bytes=dest.stat().st_size,
                          sha256=hashlib.sha256(dest.read_bytes()).hexdigest()))
gif = target / 'readme-hero.gif'
shutil.copyfile(root / 'out/readme-hero.gif', gif)
files.append(dict(path=f'previews/homepage/{gif.name}', bytes=gif.stat().st_size,
                  sha256=hashlib.sha256(gif.read_bytes()).hexdigest()))
(target / 'provenance.json').write_text(json.dumps(dict(
    source_commit=os.environ['GITHUB_SHA'],
    run_url=f'https://github.com/{os.environ["GITHUB_REPOSITORY"]}/actions/runs/{os.environ["GITHUB_RUN_ID"]}',
    files=files), indent=2) + '\n')
