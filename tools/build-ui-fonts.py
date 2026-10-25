#!/usr/bin/env python3
"""Build the UI's Google Sans Flex profiles from a pinned upstream font.

Requires fontTools. The generated static instances are committed, so device
builds and startup do not need Python, network access or variable-font shaping.
"""
import argparse
import hashlib
import io
import json
from pathlib import Path
import urllib.request

from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

ROOT = Path(__file__).resolve().parents[1]
SOURCE_COMMIT = 'bd8f81ddb5c74d5c8897b36ad88b440266245103'
SOURCE_BLOB = '1d8d8ae5a7c6d0e48312d2a0a478bad92caee73a'
SOURCE_URL = ('https://raw.githubusercontent.com/google/fonts/' + SOURCE_COMMIT +
    '/ofl/googlesansflex/GoogleSansFlex%5BROND%2Copsz%2Cslnt%2Cwdth%2Cwght%5D.ttf')
PROFILES = {
    'menu': dict(wght=520, wdth=92, opsz=30, ROND=20, slnt=0),
    'focus': dict(wght=650, wdth=96, opsz=30, ROND=30, slnt=0),
    'display': dict(wght=680, wdth=100, opsz=36, ROND=55, slnt=0),
    'value': dict(wght=600, wdth=94, opsz=24, ROND=10, slnt=0),
    'detail': dict(wght=480, wdth=100, opsz=14, ROND=0, slnt=0),
    'print': dict(wght=550, wdth=85, opsz=9, ROND=0, slnt=0),
}

def tabular_cmap(font):
    """Select the font's own tabular alternates for the simple glyph renderer."""
    substitutions = {}
    gsub = font['GSUB'].table
    for record in gsub.FeatureList.FeatureRecord:
        if record.FeatureTag != 'tnum':
            continue
        for index in record.Feature.LookupListIndex:
            for table in gsub.LookupList.Lookup[index].SubTable:
                substitutions.update(getattr(table, 'mapping', {}))
    for table in font['cmap'].tables:
        if table.isUnicode():
            for code, name in list(table.cmap.items()):
                table.cmap[code] = substitutions.get(name, name)
    widths = {font['hmtx'][font.getBestCmap()[ord(d)]][0] for d in '0123456789'}
    assert len(widths) == 1, 'Numeric profile must have equal-width digits'

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path)
    args = parser.parse_args()
    data = args.source.read_bytes() if args.source else urllib.request.urlopen(SOURCE_URL, timeout=60).read()
    blob = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
    assert blob == SOURCE_BLOB, 'Source differs from the pinned Google Fonts revision'
    output = ROOT / 'crates/slot-ui/assets/fonts'
    output.mkdir(parents=True, exist_ok=True)
    manifest = dict(family='Google Sans Flex', source_url=SOURCE_URL, source_blob=SOURCE_BLOB,
        source_sha256=hashlib.sha256(data).hexdigest(), profiles={})
    for role, axes in PROFILES.items():
        variable = TTFont(io.BytesIO(data), recalcTimestamp=False)
        font = instantiateVariableFont(variable, axes, inplace=True)
        font.recalcTimestamp = False
        if role == 'value':
            tabular_cmap(font)
        for name_id, text in {1: 'Google Sans Flex', 2: 'Slot ' + role.title(),
            4: 'Google Sans Flex Slot ' + role.title(), 6: 'GoogleSansFlex-Slot-' + role.title(),
            16: 'Google Sans Flex', 17: 'Slot ' + role.title()}.items():
            font['name'].setName(text, name_id, 3, 1, 0x409)
        font.flavor = None
        path = output / (role + '.ttf')
        font.save(path)
        manifest['profiles'][role] = dict(axes=axes, sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
            tabular=role == 'value')
        print(role, axes, path.stat().st_size)
    (output / 'profiles.json').write_text(json.dumps(manifest, indent=2) + '\n')

if __name__ == '__main__':
    main()
