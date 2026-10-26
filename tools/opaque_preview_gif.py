"""Encode actual recorder frames without a GIF transparency index.
Usage: python tools/opaque_preview_gif.py FRAMES_DIR OUTPUT.gif
Extract first with ffmpeg -i record.mp4 -vf fps=20 FRAMES_DIR/%04d.png
"""
from pathlib import Path
import argparse
from PIL import Image
parser = argparse.ArgumentParser()
parser.add_argument('frames_dir')
parser.add_argument('output')
parser.add_argument('--width', type=int, help='Downsample to the displayed README width')
parser.add_argument('--frame-ms', type=int, default=50, help='Frame interval; match the extraction rate')
args = parser.parse_args()
frames = [Image.open(p).convert('RGB') for p in sorted(Path(args.frames_dir).glob('*.png'))]
if args.width:
    frames = [f.resize((args.width, round(f.height * args.width / f.width)), Image.Resampling.LANCZOS) for f in frames]
assert frames
# Sample the complete clip when choosing a common palette.
count = min(12, len(frames))
w, h = frames[0].size
band_h = min(80, h)
sample = Image.new('RGB', (w, count * (h + 3 * band_h)))
for i in range(count):
    frame = frames[i * (len(frames)-1) // max(1, count-1)]
    sample.paste(frame, (0, i*h))
    # Give the shell enough palette entries to retain its subtle moulded relief.
    band = frame.crop((0, h-band_h, w, h))
    for repeat in range(3):
        sample.paste(band, (0, count*h + (3*i+repeat)*band_h))
palette = sample.quantize(colors=256)
encoded = [f.quantize(palette=palette, dither=Image.Dither.FLOYDSTEINBERG) for f in frames]
encoded[0].save(args.output, save_all=True, append_images=encoded[1:], duration=args.frame_ms,
                loop=0, optimize=True, disposal=1)
with Image.open(args.output) as gif:
    assert 'transparency' not in gif.info
    for i in range(gif.n_frames):
        gif.seek(i)
        assert gif.convert('RGBA').getchannel('A').getextrema() == (255,255)
print(f'Encoded {len(frames)} opaque frames')
