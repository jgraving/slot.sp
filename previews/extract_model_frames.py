"""Decode the native clip at the README GIF's 80 ms sampling interval."""
import json
from pathlib import Path
import sys
import av
source, root = map(Path, sys.argv[1:])
target = root / 'native-frames'
target.mkdir(parents=True, exist_ok=True)
with av.open(str(source)) as clip:
    duration = clip.duration / av.time_base
    count = round(duration * 1000 / 80)
    index = 0
    for frame in clip.decode(video=0):
        while index < count and frame.time + 1e-6 >= index * .08:
            frame.to_image().convert('RGB').save(target / f'{index:04}.png')
            index += 1
        if index == count:
            break
assert index == count, (index, count)
(root / 'native-frames.json').write_text(json.dumps(dict(duration=duration, frames=count)))
print('Decoded', count, 'native screen frames')
