#!/usr/bin/env python3
"""Apply the site's persisted artwork setting before accessing the catalog."""
import argparse
import json
from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parents[1]

def enabled(policy):
    value = json.loads(policy.read_text())['catalog_enabled']
    if type(value) is not bool:
        raise ValueError('catalog_enabled must be a boolean')
    return value

def clear(cache, target):
    for path in (cache, target):
        if path.exists():
            shutil.rmtree(path)
    target.mkdir(parents=True)
    (target / 'index.json').write_text('{}\n')
    (target / 'availability.json').write_text('{"catalog_enabled":false}\n')

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--policy', type=Path, default=ROOT / 'site/artwork-policy.json')
    args = parser.parse_args()
    cache = ROOT / '.studio-art-cache'
    target = ROOT / 'site/public/studio/art'
    catalog = ROOT / '.studio-art-source.json'
    if enabled(args.policy):
        raise ValueError('Hosted catalog downloads are retired; use the on-device editor')
    catalog.unlink(missing_ok=True)
    clear(cache, target)
    print('Published an empty browser catalog; on-device artwork is independent')

if __name__ == '__main__':
    main()
