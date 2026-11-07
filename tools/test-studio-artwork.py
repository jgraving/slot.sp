"""Verify that clearing a catalog removes cached and previously published images."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('artwork', Path(__file__).with_name('studio-artwork.py'))
artwork = importlib.util.module_from_spec(spec)
spec.loader.exec_module(artwork)

class Controls(unittest.TestCase):
    def test_clear_removes_nested_cached_and_published_files_and_is_repeatable(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            cache, target = root/'cache', root/'published'
            for directory in (cache, target):
                (directory/'GB/support-texture').mkdir(parents=True)
                (directory/'GB/support-texture/scan.png').write_bytes(b'old scan')
                (directory/'index.json').write_text('{"old":{}}')
            for _ in range(2):
                artwork.clear(cache, target)
                self.assertFalse(cache.exists())
                self.assertEqual(json.loads((target/'index.json').read_text()), {})
                self.assertEqual(json.loads((target/'availability.json').read_text()), {'catalog_enabled':False})
                self.assertEqual(sorted(p.name for p in target.rglob('*')), ['availability.json','index.json'])

    def test_policy_fails_closed_for_invalid_or_missing_setting(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)/'policy.json'
            for value in (True,False):
                path.write_text(json.dumps({'catalog_enabled':value}))
                self.assertIs(artwork.enabled(path), value)
            path.write_text('{"catalog_enabled":"false"}')
            with self.assertRaises(ValueError): artwork.enabled(path)
            path.write_text('{}')
            with self.assertRaises(KeyError): artwork.enabled(path)

if __name__ == '__main__':
    unittest.main()
