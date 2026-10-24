"""Prepare original label crops and a custom wallpaper for README captures."""
import json
import sys
import shutil
from pathlib import Path

root = Path(sys.argv[1])
sources = json.loads(Path(__file__).with_name('art-sources.json').read_text())
for name in sources:
    source = Path(__file__).with_name('artwork') / (name.replace(' ', '-') + '.png')
    shutil.copyfile(source, root / 'Labels/GBA' / (name + '.png'))

shutil.copyfile(Path(__file__).resolve().parents[1] / 'assets/background_default.png', root / 'Wallpapers/twilight.png')
