"""Join the native carousel and glass logo, fading back into the carousel loop."""
import json
from pathlib import Path
import subprocess
import sys

root = Path(sys.argv[1]) / 'out'
finishes = sys.argv[2:] or ['graphite', 'platinum', 'pearl_blue', 'pearl_pink']

def duration(path):
    info = json.loads(subprocess.check_output([
        'ffprobe', '-v', 'error', '-show_entries', 'format=duration',
        '-of', 'json', str(path)], text=True))
    return float(info['format']['duration'])

for finish in finishes:
    carousel = root / finish / 'hero.mp4'
    glass = root / f'hero-{finish}.mp4'
    a, b = duration(carousel), duration(glass)
    fade = .5
    joined = a + b - fade
    target = root / finish / 'combined.mp4'
    # Keep both sides of the loop inside the opening carousel pause. The second
    # crossfade restores its neighbours and status icons after the logo clears.
    filters = (
        '[0:v]fps=60,format=yuv444p,settb=1/60,setpts=PTS-STARTPTS,split=2[c0][c1];'
        '[1:v]fps=60,format=yuv444p,settb=1/60,setpts=PTS-STARTPTS[g];'
        f'[c0][g]xfade=transition=fade:duration={fade}:offset={a-fade}[first];'
        f'[first][c1]xfade=transition=fade:duration={fade}:offset={joined-fade},'
        f'trim=start={fade}:end={joined},setpts=PTS-STARTPTS[v]'
    )
    subprocess.run([
        'ffmpeg', '-nostdin', '-v', 'error', '-y', '-filter_complex_threads', '1',
        '-i', str(carousel), '-i', str(glass), '-filter_complex', filters,
        '-map', '[v]', '-an', '-c:v', 'libx264', '-threads', '2',
        '-preset', 'slow', '-crf', '20', '-pix_fmt', 'yuv420p',
        '-movflags', '+faststart', str(target)], check=True)
    assert abs(duration(target) - (a + b - 2 * fade)) < .02
    target.replace(carousel)
    print(f'{finish}: carousel, glass logo, carousel — {a+b-2*fade:.1f}s')
