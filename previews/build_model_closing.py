"""Derive matching lid-closing clips from the reviewed 3D openings."""
from fractions import Fraction
import hashlib
import json
from pathlib import Path

import av

COLORS = ['platinum', 'graphite', 'pearl_blue', 'pearl_pink', 'gold']


def reverse_clip(source, destination):
    with av.open(str(source)) as clip:
        frames = list(clip.decode(video=0))
        rate = clip.streams.video[0].average_rate
    with av.open(str(destination), 'w', options={'movflags': '+faststart'}) as output:
        stream = output.add_stream('libx264', rate=rate)
        stream.width, stream.height = frames[0].width, frames[0].height
        stream.pix_fmt = 'yuv420p'
        stream.options = {'crf': '19', 'preset': 'medium'}
        for index, original in enumerate(reversed(frames)):
            frame = original.reformat(format='yuv420p')
            frame.pts, frame.time_base = index, Fraction(1, rate)
            for packet in stream.encode(frame):
                output.mux(packet)
        for packet in stream.encode():
            output.mux(packet)


def build_closing(directory, base):
    derived = []
    for color in COLORS:
        source = directory / f'{color}-opening.mp4'
        destination = directory / f'{color}-closing.mp4'
        reverse_clip(source, destination)
        derived.append(dict(path=str(destination.relative_to(base)),
                            source=str(source.relative_to(base)),
                            source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),
                            method='Reverse the reviewed opening frames; H.264, matching frame rate'))
    return derived


if __name__ == '__main__':
    repo = Path(__file__).resolve().parents[1]
    manifest = repo / 'previews/sp-model/provenance.json'
    data = json.loads(manifest.read_text())
    directory = manifest.parent / data['revision']
    data['derived_closing'] = build_closing(directory, repo)
    data['files'] = [dict(path=str(p.relative_to(repo)), bytes=p.stat().st_size,
                          sha256=hashlib.sha256(p.read_bytes()).hexdigest())
                     for p in sorted(directory.iterdir()) if p.is_file()]
    manifest.write_text(json.dumps(data, indent=2) + '\n')
    print('Prepared five matching 3D lid-closing clips')
