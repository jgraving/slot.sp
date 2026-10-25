#!/usr/bin/env python3
"""Build website guides and media from the fork's maintained sources."""
import argparse
import hashlib
import io
import json
from pathlib import Path
import re
import shutil
import subprocess
import urllib.request

from fontTools.ttLib import TTFont
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
DOCS = ROOT / 'site/docs'
GUIDES = ['GUIDE', 'GETTING-STARTED', 'CONTROLS', 'LIBRARY', 'PLAYING',
          'SCREENSHOTS', 'DISPLAY', 'NIGHT-DISPLAY', 'BLUETOOTH', 'KID-MODE',
          'POWER-OPTIONS', 'LINK-PLAY']

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--font-source', type=Path)
    args = parser.parse_args()
    features = DOCS / 'features'
    media = DOCS / 'media/fork'
    fonts = DOCS / 'assets/fonts'
    for directory in [features, media, fonts]:
        directory.mkdir(parents=True, exist_ok=True)
    mapping = {name + '.md': name.lower() + '.md' for name in GUIDES}
    mapping['README.md'] = '../index.md'
    def link(match):
        path, anchor = match.group(1), match.group(2) or ''
        if not path or path in mapping or path.startswith('previews/'):
            anchor = re.sub(r'-{2,}', '-', anchor)
        if path in mapping:
            path = mapping[path]
        elif path.startswith('previews/'):
            path = '../media/fork/' + Path(path).name
        return '](' + path + anchor + ')'
    for name in GUIDES:
        content = (ROOT / (name + '.md')).read_text()
        content = re.sub(r'\]\(([^)#]*)(#[^)]*)?\)', link, content)
        # Render GitHub callouts as native site admonitions.
        content = re.sub(r'^> \[!(TIP|NOTE|WARNING)\]\n((?:>.*\n?)+)',
            lambda m: '!!! ' + m[1].lower() + '\n\n' + ''.join(
                '    ' + line.removeprefix('> ').removeprefix('>') + '\n'
                for line in m[2].splitlines()), content, flags=re.M)
        (features / (name.lower() + '.md')).write_text(content)
    shutil.copyfile(ROOT / 'previews/rgsp-shell-finishes.png', media / 'rgsp-shell-finishes.png')
    shutil.copyfile(ROOT / 'previews/hero-glass.mp4', media / 'brand.mp4')
    Image.open(ROOT / 'previews/hero-glass.png').save(media / 'brand.webp', quality=85)
    for name in ['carousel', 'core-picker', 'link', 'shutdown']:
        source = ROOT / f'previews/readme-{name}.gif'
        subprocess.run(['ffmpeg', '-nostdin', '-hide_banner', '-loglevel', 'error', '-y',
            '-i', str(source), '-an', '-vf', 'fps=30,scale=720:480:flags=lanczos',
            '-c:v', 'libx264', '-pix_fmt', 'yuv420p', '-crf', '24',
            '-movflags', '+faststart', str(media / (name + '.mp4'))], check=True)
        Image.open(source).convert('RGB').save(media / (name + '.webp'), quality=85)
    manifest = json.loads((ROOT / 'crates/slot-ui/assets/fonts/profiles.json').read_text())
    data = args.font_source.read_bytes() if args.font_source else urllib.request.urlopen(
        manifest['source_url'], timeout=60).read()
    assert hashlib.sha256(data).hexdigest() == manifest['source_sha256'], 'Font source hash differs'
    font = TTFont(io.BytesIO(data), recalcTimestamp=False)
    font.flavor = 'woff2'
    font.save(fonts / 'google-sans-flex.woff2')
    shutil.copyfile(ROOT / 'licenses/GoogleSansFlex-OFL.txt', fonts / 'OFL.txt')
    print(f'Prepared {len(GUIDES)} guides, fork previews and self-hosted Google Sans Flex')

if __name__ == '__main__':
    main()
