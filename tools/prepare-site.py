#!/usr/bin/env python3
"""Build website guides and media from the fork's maintained sources."""
import argparse
import hashlib
import io
import json
from pathlib import Path
import re
import shutil
import urllib.request

from fontTools.ttLib import TTFont

ROOT = Path(__file__).resolve().parents[1]
DOCS = ROOT / 'site/docs'
GUIDES = ['GUIDE', 'GETTING-STARTED', 'CONTROLS', 'LIBRARY', 'PLAYING',
          'SCREENSHOTS', 'DISPLAY', 'NIGHT-DISPLAY', 'BLUETOOTH', 'KID-MODE',
          'POWER-OPTIONS', 'LINK-PLAY', 'CART-STUDIO']

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
        # The website has a sidebar and section links; GitHub keeps its own navigation.
        content = re.sub(r'^# [^A-Za-z0-9]+', '# ', content, count=1)
        lines = content.splitlines()
        lines = [line for line in lines if not line.startswith('[←')
                 and not (line.startswith('[') and '](' in line and '(#' in line)]
        content = '\n'.join(lines) + '\n'
        content = content.replace('<details>', '<details markdown>')
        content = re.sub(r'\]\(([^)#]*)(#[^)]*)?\)', link, content)
        def video(match):
            label, stem = match.groups()
            stem = {'slot-fit-motion': 'cart-motion', 'power-switch': 'readme-shutdown'}.get(stem, stem)
            return (f'<video controls muted playsinline loop preload="none" width="720" height="480" '
                    f'poster="/slot.sp/media/fork/{stem}.png" aria-label="{label}">'
                    f'<source src="/slot.sp/media/fork/{stem}.mp4" type="video/mp4"></video>')
        content = re.sub(r'!\[([^]]*)\]\(../media/fork/([^/)]+)\.gif\)', video, content)
        # Render GitHub callouts as native site admonitions.
        content = re.sub(r'^> \[!(TIP|NOTE|WARNING)\]\n((?:>.*\n?)+)',
            lambda m: '!!! ' + m[1].lower() + '\n\n' + ''.join(
                '    ' + line.removeprefix('> ').removeprefix('>') + '\n'
                for line in m[2].splitlines()), content, flags=re.M)
        (features / (name.lower() + '.md')).write_text(content)
    provenance = json.loads((ROOT / 'previews/media-provenance.json').read_text())
    # Publish the reviewed captures, not obsolete experiments in the source folder.
    for entry in provenance['files']:
        source = ROOT / entry['path']
        assert hashlib.sha256(source.read_bytes()).hexdigest() == entry['sha256'], source
        if source.suffix in ('.mp4', '.png'):
            shutil.copyfile(source, media / source.name)
    shutil.copyfile(ROOT / 'previews/rgsp-shell-finishes.png', media / 'rgsp-shell-finishes.png')
    shutil.copyfile(ROOT / 'previews/media-provenance.json', media / 'media-provenance.json')
    homepage = json.loads((ROOT / 'previews/homepage/provenance.json').read_text())
    revision = homepage['source_commit'][:12]
    homepage_media = media / 'homepage'
    if homepage_media.exists():
        shutil.rmtree(homepage_media)
    for entry in homepage['files']:
        source = ROOT / entry['path']
        assert hashlib.sha256(source.read_bytes()).hexdigest() == entry['sha256'], source
        target = homepage_media / revision / source.name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
    shutil.copyfile(ROOT / 'previews/homepage/provenance.json', homepage_media / 'provenance.json')
    manifest = json.loads((ROOT / 'crates/slot-ui/assets/fonts/profiles.json').read_text())
    data = args.font_source.read_bytes() if args.font_source else urllib.request.urlopen(
        manifest['source_url'], timeout=60).read()
    assert hashlib.sha256(data).hexdigest() == manifest['source_sha256'], 'Font source hash differs'
    font = TTFont(io.BytesIO(data), recalcTimestamp=False)
    font.flavor = 'woff2'
    font.save(fonts / 'google-sans-flex.woff2')
    shutil.copyfile(ROOT / 'licenses/GoogleSansFlex-OFL.txt', fonts / 'OFL.txt')
    # Fingerprint the app styles and script so a cached previous version cannot
    # be combined with a freshly deployed page. Keep source config unchanged.
    generated = DOCS / 'assets/generated'
    if generated.exists():
        shutil.rmtree(generated)
    generated.mkdir(parents=True)
    config = (ROOT / 'site/zensical.toml').read_text()
    config = config.replace('homepage_media_revision = "prepared"',
                            f'homepage_media_revision = "{revision}"')
    for relative in ['assets/device.css', 'assets/slot.css', 'assets/slot.js']:
        source = DOCS / relative
        content = source.read_text()
        if source.suffix == '.css':
            # Generated CSS is one directory deeper than the original file.
            def parent_url(match):
                raw = match[1].strip()
                quote = raw[0] if raw.startswith(('"', "'")) else ''
                path = raw[1:-1] if quote else raw
                if path.startswith(('/', '#')) or re.match(r'[a-z]+:', path):
                    return match[0]
                return f'url({quote}../{path}{quote})'
            content = re.sub(r'url\(([^)]+)\)', parent_url, content)
        digest = hashlib.sha256(content.encode()).hexdigest()[:12]
        name = f'{source.stem}.{digest}{source.suffix}'
        (generated / name).write_text(content)
        config = config.replace('"' + relative + '"', '"assets/generated/' + name + '"')
    (ROOT / 'site/zensical.build.toml').write_text(config)
    print(f'Prepared {len(GUIDES)} guides, reviewed media, font and fingerprinted website assets')

if __name__ == '__main__':
    main()
