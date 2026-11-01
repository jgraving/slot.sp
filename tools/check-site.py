#!/usr/bin/env python3
"""Check built website links, anchors and local media before Pages deployment."""
from html.parser import HTMLParser
import av
from pathlib import Path
import json
import re
from PIL import Image
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
        for candidate in attrs.get('srcset','').split(','):
            if candidate.strip():self.links.append((tag,candidate.strip().split()[0]))
        for link in re.findall(r'url\([\"\']?([^\)\"\']+)',attrs.get('style','')):
            self.links.append((tag,link))
        for key in ('href', 'src', 'poster', 'data-src'):
            if attrs.get(key):
                self.links.append((tag, attrs[key]))

pages = {path: Page(path.read_text()) for path in ROOT.rglob('*.html')}
dynamic_ids = set()
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
for source in list((ROOT / 'assets').rglob('*.css')) + list((ROOT / 'studio').rglob('*.css')):
    for link in re.findall(r'url\([\"\']?([^\)\"\']+)', source.read_text()):
        if not urlsplit(link).scheme and not (source.parent / link).is_file():
            errors.append(f'{source.name}: missing CSS asset {link}')
        checked += 1
home = (ROOT / 'index.html').read_text()
assert 'https://jakegraving.com/slot.sp/' in home
assert 'Your games.' in home and 'features/kid-mode/' in home
assert not re.search(r'game\s*boy', home, re.I), 'Keep third-party system names out of homepage text'
assert 'BrandonKowalski/slot/releases' not in home
assert 'fonts.googleapis.com' not in home, 'Typeface must be self-hosted'
assert not (ROOT / 'CNAME').exists(), 'Do not publish the upstream custom domain'
studio_pin = json.loads((ROOT.parent / 'studio.json').read_text())
studio_info = json.loads((ROOT / 'studio/build-info.json').read_text())
assert studio_info['source_commit'] == studio_pin['ref']
assert studio_info['repository'] == 'jgraving/slot-cart-studio'
assert studio_info['renderer_repository'] == 'jgraving/slot.sp'
assert studio_info['assets_base'] == f"assets/{studio_info['source_commit'][:12]}/"
assert (ROOT / 'studio' / studio_info['assets_base'] / 'pkg/slot_cart_studio_bg.wasm').read_bytes()[:8] == b'\0asm\x01\0\0\0'
assert (ROOT / 'studio/LICENSE').is_file()
hero_durations = []
hero_manifest = json.loads((ROOT / 'media/fork/homepage/provenance.json').read_text())
hero_revision = hero_manifest.get('capture_source_commit',hero_manifest['source_commit'])[:12]
for theme in ['dark','light']:
    for finish in ['graphite','platinum','pearl_blue','pearl_pink']:
        stem=ROOT/f'media/fork/homepage/{hero_revision}/hero-{theme}-{finish}'
        with Image.open(stem.with_suffix('.png')) as poster:assert poster.size==(720,480),stem
        with av.open(str(stem.with_suffix('.mp4'))) as clip:hero_durations.append(clip.duration)
assert len(set(hero_durations))==1,'Theme and finish previews must share a timeline'
current=json.loads((ROOT/'media/fork/current/provenance.json').read_text())
revision=current.get('capture_source_commit',current['source_commit'])[:12]
for theme in ['dark','light']:
    for scene in ['carousel','settings','general','screen','sound','parent-settings','parent-code','kid-menu','in-game-menu','in-game-settings','charging','boot','power','controls','about','screenshots','save-states','buttons-shortcuts','kid-warning','core-picker','approved','favorites','link-host','link-join']:
        if theme=='light' and scene in ['save-states','link-host','link-join']:continue
        path=ROOT/f'media/fork/current/{revision}/{theme}/{scene}.png'
        with Image.open(path) as image:assert image.size==(720,480),path
    for scene in ['boot','settings','parent-code','charging','carousel']:
        assert (ROOT/f'media/fork/current/{revision}/{theme}/{scene}.mp4').is_file(),scene
assert 'site-theme-toggle' not in home and 'ui-theme-picker' not in home
assert 'detail-pair' not in home and 'motion-section' not in home
assert home.count('<video') == 1, 'Keep homepage media inside the illustrated SP preview'
assert f'hero-dark-platinum.mp4' in home
assert hero_manifest['sequence']==['insert','play','eject','browse','boot']
assert all(t['insert_frame']==48 and t['visible_insertion_frames']>=24 for t in hero_manifest['timelines'])
for theme in ['dark','light']:
    for scene in ['screenshots-full','screenshots-zoom','screenshots-select']:
        with Image.open(ROOT/f'media/fork/current/{revision}/{theme}/{scene}.png') as image:assert image.size==(720,480)
for video in (ROOT / 'media/fork').rglob('*.mp4'):
    with av.open(str(video)) as container:
        assert container.duration and container.duration > 0, video
        assert any(s.codec_context.name == 'h264'
                   and s.codec_context.width == 720 and s.codec_context.height == 480
                   for s in container.streams.video), video
        frame = next(container.decode(video=0))
        assert (frame.width, frame.height) == (720, 480), video

if errors:
    raise SystemExit('\n'.join(sorted(set(errors))))
print(f'PASS: {len(pages)} pages; {checked} local links, anchors and assets; project base {BASE}')

# Artwork is served from the same origin; browser CORS must not gate real labels.
art_root=ROOT/'studio/art'
site_bytes = sum(path.stat().st_size for path in ROOT.rglob('*') if path.is_file())
assert site_bytes <= 950 * 1024 * 1024, f'Website exceeds 950 MiB deployment budget: {site_bytes} bytes'
print(f'Website size: {site_bytes / 1024 / 1024:.1f} MiB (950 MiB budget)')
art_index=json.loads((art_root/'index.json').read_text())
assert art_index, 'Studio artwork catalog is empty'
for entry in art_index.values():
    for kind in ('support-texture','wheel'):
        if name:=entry.get(kind):
            path=art_root/name
            assert path.is_file() and path.read_bytes()[:8]==b'\x89PNG\r\n\x1a\n', name
