"""Replace only the reviewed audio examples, preserving other capture identities."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import sys

repo = Path(__file__).resolve().parents[1]
root = Path(sys.argv[1])
target = root / 'publish/previews/current'
target.mkdir(parents=True, exist_ok=True)
manifest = json.loads((repo / 'previews/current/provenance.json').read_text())
source = os.environ['GITHUB_SHA']
for entry in manifest['files']:
    entry.setdefault('capture_source_commit', manifest.get('capture_source_commit', manifest['source_commit']))
    entry.setdefault('ui_source_commit', manifest.get('ui_release_commit', manifest['source_commit']))
for theme in ['dark', 'light']:
    for scene in ['sound', 'sound-pair', 'sound-options']:
        relative = f'previews/current/{theme}/{scene}.png'
        path = root / 'publish' / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(root / 'raw/current' / theme / f'{scene}.png', path)
        manifest['files'] = [e for e in manifest['files'] if e['path'] != relative]
        manifest['files'].append(dict(path=relative, bytes=path.stat().st_size,
            sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
            capture_source_commit=source, ui_source_commit=source))
manifest.update(capture_source_commit=source, refresh_scenes=['sound', 'sound-pair', 'sound-options'],
    run_url=f'https://github.com/{os.environ["GITHUB_REPOSITORY"]}/actions/runs/{os.environ["GITHUB_RUN_ID"]}')
manifest['audio_fixture'] = 'Bluetooth-capable handheld, enabled radio and a synthetic saved Headphones device. Production Frontend/compositor; no physical hardware pairing.'
(target / 'provenance.json').write_text(json.dumps(manifest, indent=2) + '\n')
