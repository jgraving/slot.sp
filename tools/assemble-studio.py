#!/usr/bin/env python3
"""Publish the pinned Studio app and renderer together under versioned asset URLs."""
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
assets_base = f'assets/{commit[:12]}/'
bundle = target / assets_base
shutil.copytree(source / 'web', bundle, dirs_exist_ok=True,
                ignore=shutil.ignore_patterns('index.html', '*.test.mjs', '.gitignore'))
script = bundle / 'studio.js'
text, count = re.subn(r"^const BUILD = 'dev';$", f"const BUILD = '{commit}';", script.read_text(), flags=re.M)
assert count == 1, 'Studio build stamp is missing'
script.write_text(text)
# Studio CSS previously lived one directory below the website's font assets.
for css in bundle.glob('*.css'):
    css.write_text(css.read_text().replace('../assets/fonts/', '/slot.sp/assets/fonts/'))
html = (source / 'web/index.html').read_text()
for name in ['studio.js', 'studio.css', 'slot-sp.css']:
    html = html.replace(f'"{name}"', f'"{assets_base}{name}"')
html = html.replace('<p id="build"></p>', '<p><a href="/slot.sp/impressum/">Impressum</a> · <a href="/slot.sp/datenschutz/">Datenschutz</a></p><p id="build"></p>')
(target / 'index.html').write_text(html)
shutil.copyfile(source / 'LICENSE', target / 'LICENSE')
(target / 'build-info.json').write_text(json.dumps(dict(
    repository=pin['repository'], source_commit=commit, assets_base=assets_base,
    renderer_repository=(source / 'slot.repo').read_text().strip(),
    renderer_commit=renderer_commit), indent=2) + '\n')
print(f'Assembled Cart Studio {commit} with versioned assets at /slot.sp/studio/')
