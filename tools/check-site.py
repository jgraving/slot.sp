#!/usr/bin/env python3
"""Check built website links, anchors and local media before Pages deployment."""
from html.parser import HTMLParser
from pathlib import Path
import json
import re
import subprocess
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1] / 'site/public'
ORIGIN = 'jakegraving.com'
BASE = '/slot.sp/'

class Page(HTMLParser):
    def __init__(self, source):
        super().__init__()
        self.ids = set()
        self.links = []
        self.feed(source)

    def handle_starttag(self, tag, attributes):
        attrs = dict(attributes)
        if attrs.get('id'):
            self.ids.add(attrs['id'])
        for key in ('href', 'src', 'poster', 'data-src'):
            if attrs.get(key):
                self.links.append((tag, attrs[key]))

pages = {path: Page(path.read_text()) for path in ROOT.rglob('*.html')}
dynamic_ids = set(re.findall(r'id="([^"]+)"', (ROOT / 'assets/device.svg').read_text()))
errors = []
checked = 0
for source, page in pages.items():
    for tag, link in page.links:
        url = urlsplit(link)
        if url.netloc and url.netloc != ORIGIN or url.scheme and url.scheme not in ('http', 'https'):
            continue
        path = unquote(url.path)
        if path.startswith('/'):
            if not path.startswith(BASE):
                errors.append(f'{source.relative_to(ROOT)}: outside project base: {link}')
                continue
            target = ROOT / path[len(BASE):]
        else:
            target = source.parent / path if path else source
        target = target.resolve()
        if target.is_dir():
            target /= 'index.html'
        if not target.is_file():
            errors.append(f'{source.relative_to(ROOT)}: missing {link}')
        elif url.fragment and target in pages and unquote(url.fragment) not in pages[target].ids:
            if not (tag == 'use' and url.fragment in dynamic_ids):
                errors.append(f'{source.relative_to(ROOT)}: missing anchor {link}')
        checked += 1
for source in list((ROOT / 'assets').glob('*.css')) + list((ROOT / 'studio').glob('*.css')):
    for link in re.findall(r'url\([\"\']?([^\)\"\']+)', source.read_text()):
        if not urlsplit(link).scheme and not (source.parent / link).is_file():
            errors.append(f'{source.name}: missing CSS asset {link}')
        checked += 1
home = (ROOT / 'index.html').read_text()
assert 'https://jakegraving.com/slot.sp/' in home
assert 'Game Boy games.' in home and 'features/kid-mode/' in home
assert 'BrandonKowalski/slot/releases' not in home
assert 'fonts.googleapis.com' not in home, 'Typeface must be self-hosted'
assert not (ROOT / 'CNAME').exists(), 'Do not publish the upstream custom domain'
studio_pin = json.loads((ROOT.parent / 'studio.json').read_text())
studio_info = json.loads((ROOT / 'studio/build-info.json').read_text())
assert studio_info['source_commit'] == studio_pin['ref']
assert studio_info['repository'] == 'jgraving/slot-cart-studio'
assert (ROOT / 'studio/pkg/slot_cart_studio_bg.wasm').read_bytes()[:8] == b'\0asm\x01\0\0\0'
assert (ROOT / 'studio/LICENSE').is_file()
for video in (ROOT / 'media/fork').glob('*.mp4'):
    metadata = json.loads(subprocess.check_output(['ffprobe', '-v', 'error',
        '-show_entries', 'stream=codec_name,width,height:format=duration', '-of', 'json', str(video)]))
    assert float(metadata['format']['duration']) > 0, video
    assert any(s.get('codec_name') == 'h264' and s.get('width') == 720
               and s.get('height') == 480 for s in metadata['streams']), video
if errors:
    raise SystemExit('\n'.join(sorted(set(errors))))
print(f'PASS: {len(pages)} pages; {checked} local links, anchors and assets; project base {BASE}')
