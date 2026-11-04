#!/usr/bin/env python3
"""Exercise rebuild safety without downloading or compiling Samba."""
import importlib.util
from pathlib import Path
import tempfile
import unittest


spec = importlib.util.spec_from_file_location('prepare_samba_output', Path(__file__).with_name('prepare-samba-output.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class RuntimeOutputTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.output = self.root / 'runtime'
        self.sources = self.root / 'sources'
        self.cache = self.root / 'build-cache'

    def prepare(self):
        module.prepare(self.output, self.sources, self.cache)

    def test_rebuild_discards_stale_libraries_gconv_and_notices_but_keeps_sources(self):
        self.prepare()
        for name in ('lib/libc.so.6', 'gconv/IBM850.so', 'gconv/gconv/IBM850.so', 'licenses/dependency-sources.tsv'):
            path = self.output / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b'stale')
        (self.output / 'build-info.txt').write_text('Samba 4.25.1\nsource_sha256=' + 'a' * 64 + '\nwsdd2_source_sha256=' + 'b' * 64 + '\n')
        (self.sources / 'glibc.orig.tar.xz').write_bytes(b'source')
        (self.cache / 'samba.tar.gz').write_bytes(b'download')
        self.prepare()
        self.assertEqual(set(path.name for path in self.output.iterdir()), {'build-info.txt'})
        self.assertEqual((self.sources / 'glibc.orig.tar.xz').read_bytes(), b'source')
        self.assertEqual((self.cache / 'samba.tar.gz').read_bytes(), b'download')
        # Fresh copies cannot be skipped because an old dependency already exists.
        self.assertFalse((self.output / 'lib/libc.so.6').exists())

    def test_interrupted_build_can_be_retried(self):
        self.prepare()
        (self.output / 'bin').mkdir()
        (self.output / 'bin/smbd').write_bytes(b'partial')
        self.prepare()
        self.assertFalse((self.output / 'bin').exists())

    def test_unrelated_directory_is_untouched(self):
        self.output.mkdir()
        precious = self.output / 'project.txt'
        precious.write_bytes(b'keep')
        with self.assertRaises(ValueError):
            self.prepare()
        self.assertEqual(precious.read_bytes(), b'keep')

    def test_unexpected_file_in_known_output_is_untouched(self):
        self.prepare()
        precious = self.output / 'project.txt'
        precious.write_bytes(b'keep')
        with self.assertRaises(ValueError):
            self.prepare()
        self.assertEqual(precious.read_bytes(), b'keep')

    def test_symlinked_output_is_rejected_and_internal_links_are_not_followed(self):
        self.prepare()
        precious = self.sources / 'source.tar'
        precious.write_bytes(b'keep')
        (self.output / 'lib').symlink_to(self.sources, target_is_directory=True)
        self.prepare()
        self.assertEqual(precious.read_bytes(), b'keep')
        linked = self.root / 'alias'
        linked.symlink_to(self.output, target_is_directory=True)
        with self.assertRaises(ValueError):
            module.prepare(linked, self.sources, self.cache)
        with self.assertRaises(ValueError):
            module.prepare(linked / 'nested', self.sources, self.cache)

    def test_overlapping_paths_are_rejected_before_changes(self):
        self.prepare()
        precious = self.output / 'build-info.txt'
        before = precious.read_bytes()
        for sources, cache in ((self.output, self.cache), (self.output / 'sources', self.cache),
                               (self.sources, self.output / 'build'), (self.root, self.cache)):
            with self.assertRaises(ValueError):
                module.prepare(self.output, sources, cache)
            self.assertEqual(precious.read_bytes(), before)


if __name__ == '__main__':
    unittest.main()
