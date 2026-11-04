#!/usr/bin/env python3
"""Reset only a recognized generated runtime; retain build and source caches."""
import argparse
from pathlib import Path
import re
import shutil


ENTRIES = {'bin', 'lib', 'libexec', 'gconv', 'licenses', 'build-info.txt'}
IN_PROGRESS = 'Samba runtime build in progress\n'


def prepare(output: Path, sources: Path, cache: Path) -> None:
    # Do not resolve an output symlink and then remove its target's contents.
    absolute = output.absolute()
    if any(path.is_symlink() for path in (absolute, *absolute.parents)):
        raise ValueError('runtime output must not use a symlinked path')
    output, sources, cache = (path.resolve() for path in (output, sources, cache))
    roots = (output, sources, cache)
    if any(a == b or a in b.parents or b in a.parents
           for i, a in enumerate(roots) for b in roots[i + 1:]):
        raise ValueError('runtime output, sources and build cache must be separate trees')
    if output.exists():
        if not output.is_dir():
            raise ValueError('runtime output must be a directory')
        entries = set(path.name for path in output.iterdir())
        if entries:
            marker = output / 'build-info.txt'
            text = marker.read_text() if marker.is_file() and not marker.is_symlink() else ''
            known = text == IN_PROGRESS or re.fullmatch(
                r'Samba [0-9.]+\nsource_sha256=[0-9a-f]{64}\nwsdd2_source_sha256=[0-9a-f]{64}\n', text)
            if not known or not entries <= ENTRIES:
                raise ValueError('refusing to clean an unrecognized runtime output directory')
        for path in output.iterdir():
            if path.is_dir() and not path.is_symlink():
                shutil.rmtree(path)
            else:
                path.unlink()
    output.mkdir(parents=True, exist_ok=True)
    # Also recognizes interrupted builds, so a failed build can be retried.
    (output / 'build-info.txt').write_text(IN_PROGRESS)
    sources.mkdir(parents=True, exist_ok=True)
    cache.mkdir(parents=True, exist_ok=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('sources', type=Path)
    parser.add_argument('cache', type=Path)
    args = parser.parse_args()
    prepare(args.output, args.sources, args.cache)
