#!/usr/bin/env python3
"""Place the pinned Cart Studio build inside the Pages website."""
import json
from pathlib import Path
import re
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
pin = json.loads((ROOT / 'site/studio.json').read_text())
source = ROOT / '_studio/slot-cart-studio'
renderer = source.parent / 'slot'
commit = subprocess.check_output(['git', '-C', str(source), 'rev-parse', 'HEAD'], text=True).strip()
renderer_commit = subprocess.check_output(['git', '-C', str(renderer), 'rev-parse', 'HEAD'], text=True).strip()
assert commit == pin['ref'], (commit, pin['ref'])
assert renderer_commit == (source / 'slot.ref').read_text().strip()
target = ROOT / 'site/public/studio'
shutil.copytree(source / 'web', target, dirs_exist_ok=True,
                ignore=shutil.ignore_patterns('*.test.mjs', '.gitignore'))
script = target / 'studio.js'
text, count = re.subn(r"^const BUILD = 'dev';$", f"const BUILD = '{commit}';", script.read_text(), flags=re.M)
assert count == 1, 'Studio build stamp is missing'
script.write_text(text)
shutil.copyfile(source / 'LICENSE', target / 'LICENSE')
(target / 'build-info.json').write_text(json.dumps(dict(
    repository=pin['repository'], source_commit=commit,
    renderer_repository='BrandonKowalski/slot', renderer_commit=renderer_commit), indent=2) + '\n')
print(f'Assembled Cart Studio {commit} at /slot.sp/studio/')
