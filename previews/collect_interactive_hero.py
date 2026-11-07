"""Package verified, frame-addressed actions from the native UI recorder."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import sys

import av
import numpy as np
from PIL import Image, ImageDraw

root = Path(sys.argv[1])
out = root / 'publish/previews/interactive'
review = root / 'review'
out.mkdir(parents=True, exist_ok=True)
review.mkdir(parents=True, exist_ok=True)
names = ['Fern', 'Orbit', 'Rosette', 'Tide', 'Wander']
files = []
timelines = {}
checks = []
for finish in ['graphite', 'platinum', 'pearl_blue', 'pearl_pink']:
    source = root / 'raw' / f'{finish}.mp4'
    data = json.loads((source.with_suffix('.json')).read_text())
    segments = data['segments']
    wanted = {frame for pair in segments.values() for frame in pair}
    saved = {}
    # Only retain endpoint pixels. Each film is decoded once, using bounded memory.
    with av.open(source) as video:
        for index, frame in enumerate(video.decode(video=0)):
            if index in wanted:
                saved[index] = frame.to_ndarray(format='rgb24')
    assert index + 1 == data['frames']
    def delta(a, b):
        return float(np.abs(saved[a].astype(np.int16) - saved[b]).mean())
    def join(a, b):
        difference = delta(a, b)
        # H.264 independently quantizes the same stationary frame at each scene.
        assert difference < 3.0, (finish, 'Discontinuous native pose', a, b, difference)
        checks.append(dict(finish=finish, frames=[a, b], mean_rgb_difference=round(difference, 4)))
    join(segments['boot'][1], segments['insert-0'][0])
    for i in range(len(names)):
        start, seated = segments[f'insert-{i}']
        join(seated, segments[f'eject-{i}'][0])
        join(start, segments[f'eject-{i}'][1])
        join(start, segments[f'right-{i}'][0])
        join(segments[f'right-{i}'][1], segments[f'insert-{(i + 1) % len(names)}'][0])
        join(segments[f'left-{i}'][1], segments[f'insert-{(i - 1) % len(names)}'][0])
    # Trim the native ejection hold: the website has no game to shut down.
    # Keep the canonical seated first frame and start before the physical pop.
    for key, bounds in segments.items():
        if key.startswith('eject-'):
            bounds[0] += 19
    data.update(carts=names)
    timelines[finish] = data
    shutil.copyfile(source, out / source.name)
    # A compact strip provides immediate settled frames, reduced-motion controls,
    # and a stable old frame while the decoder seeks to the next action.
    strip = Image.new('RGB', (720, 480 * 11))
    strip.paste(Image.fromarray(saved[segments['boot'][0]]), (0, 0))
    for i in range(len(names)):
        for row, endpoint in [(1 + i, 0), (6 + i, 1)]:
            strip.paste(Image.fromarray(saved[segments[f'insert-{i}'][endpoint]]), (0, row * 480))
    strip.save(out / f'{finish}.webp', quality=88, method=6)
    Image.fromarray(saved[segments['insert-0'][0]]).save(out / f'{finish}.png', optimize=True)
    contact = Image.new('RGB', (720, 480 * 3))
    for row, i in enumerate([0, 1, 4]):
        im = Image.fromarray(saved[segments[f'insert-{i}'][0]])
        contact.paste(im, (0, row * 480))
    contact.save(review / f'{finish}.jpg', quality=92)
assert len({json.dumps(t['segments'], sort_keys=True) for t in timelines.values()}) == 1
timeline = next(iter(timelines.values()))
(out / 'timeline.json').write_text(json.dumps(timeline, separators=(',', ':')) + '\n')
for path in sorted(out.iterdir()):
    files.append(dict(path='previews/interactive/' + path.name, bytes=path.stat().st_size,
                      sha256=hashlib.sha256(path.read_bytes()).hexdigest()))
manifest = dict(source_commit=os.environ['GITHUB_SHA'],
                run_url=f'https://github.com/{os.environ["GITHUB_REPOSITORY"]}/actions/runs/{os.environ["GITHUB_RUN_ID"]}',
                capture='previews/interactive_hero.rs', dimensions=[720, 480], fps=60,
                rendering='Production Frontend, carousel springs, slot occlusion and boot compositor. Recorder-only App stepping omits emulator execution and fixes the wallpaper phase for continuous action endpoints. Synthetic headers and repository sample artwork; no playable games.',
                files=files)
(out / 'provenance.json').write_text(json.dumps(manifest, indent=2) + '\n')
(review / 'joins.json').write_text(json.dumps(checks, indent=2) + '\n')
size = sum(f['bytes'] for f in files)
assert size < 12 * 1024 * 1024, size
print(f'PASS: {len(checks)} continuous native action joins; {size / 1024 / 1024:.2f} MiB across four finishes')
