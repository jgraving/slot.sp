"""Encode actual recorder frames without a GIF transparency index.
Usage: python tools/opaque_preview_gif.py FRAMES_DIR OUTPUT.gif
Extract first with ffmpeg -i record.mp4 -vf fps=20 FRAMES_DIR/%04d.png
"""
from pathlib import Path
import sys
from PIL import Image
frames = [Image.open(p).convert('RGB') for p in sorted(Path(sys.argv[1]).glob('*.png'))]
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
encoded[0].save(sys.argv[2], save_all=True, append_images=encoded[1:], duration=50,
                loop=0, optimize=False, disposal=1)
with Image.open(sys.argv[2]) as gif:
    assert 'transparency' not in gif.info
    for i in range(gif.n_frames):
        gif.seek(i)
        assert gif.convert('RGBA').getchannel('A').getextrema() == (255,255)
print(f'Encoded {len(frames)} opaque frames')
