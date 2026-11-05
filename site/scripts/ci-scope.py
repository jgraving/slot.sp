#!/usr/bin/env python3
"""Select website checks from actual site inputs, independently of device code."""
import ast
import os
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[2]


def guides():
    for node in ast.parse((ROOT / 'tools/prepare-site.py').read_text()).body:
        if isinstance(node, ast.Assign) and any(isinstance(t, ast.Name) and t.id == 'GUIDES' for t in node.targets):
            return {name + '.md' for name in ast.literal_eval(node.value)}
    raise RuntimeError('prepare-site.py GUIDE inputs not found')


SHARED = {
    '.github/workflows/pages.yml', 'site/scripts/ci-scope.py', 'site/scripts/test-ci-scope.py',
    'tools/prepare-site.py', 'crates/slot-ui/assets/fonts/profiles.json',
    'licenses/GoogleSansFlex-OFL.txt', 'site/zensical.toml',
}

MODEL_FILES = {
    'site/docs/index.md', 'site/overrides/home.html', 'site/overrides/main.html',
    'site/overrides/partials/page-motion.js',
    'site/docs/assets/sp-model.js', 'site/docs/assets/hero-player.js',
    'site/docs/assets/device.css', 'site/docs/assets/slot.css',
    'site/docs/assets/header.css', 'site/docs/assets/page-motion.css',
}


def classify(paths, force=False):
    paths = set(paths)
    website = force or bool(paths & (guides() | SHARED)) or any(
        p.startswith(('site/', 'previews/')) or p in {
            'tools/assemble-studio.py', 'tools/check-site.py',
            'tools/check-site-navigation.cjs', 'tools/check-page-motion.cjs',
        } for p in paths)
    model = force or bool(paths & (SHARED | MODEL_FILES)) or any(
        p.startswith((
            'site/docs/assets/vendor/', 'site/docs/media/',
            'site/scripts/check-sp-', 'previews/sp-model/',
            'previews/interactive/', 'previews/homepage/',
        )) for p in paths)
    return bool(website), bool(model)


def changed_paths(base, head):
    if not re.fullmatch(r'[0-9a-fA-F]{40,64}', head or ''):
        raise ValueError('HEAD_SHA must be a commit SHA')
    if not base or set(base) == {'0'}:
        command = ['git', 'ls-files', '-z']
    else:
        if not re.fullmatch(r'[0-9a-fA-F]{40,64}', base):
            raise ValueError('BASE_SHA must be a commit SHA')
        command = ['git', 'diff', '--name-only', '-z', base, head, '--']
    return subprocess.check_output(command, cwd=ROOT).decode().rstrip('\0').split('\0')


def main():
    force = os.environ.get('FORCE_WEBSITE_CHECKS') == 'true'
    paths = [] if force else changed_paths(os.environ.get('BASE_SHA'), os.environ.get('HEAD_SHA'))
    website, model = classify(paths, force)
    output = f'website={str(website).lower()}\nmodel={str(model).lower()}\n'
    print(output, end='')
    with open(os.environ['GITHUB_OUTPUT'], 'a') as stream:
        stream.write(output)


if __name__ == '__main__':
    main()
