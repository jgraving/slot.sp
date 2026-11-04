#!/usr/bin/env python3
"""Exercise the real boot installer with fault injection, never a device."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

INSTALLER = Path(__file__).resolve().parent / 'System/slotupdate.sh'
FILES = ['slot', 'launch_frontend.sh', 'mgba_libretro.so', 'gpsp_libretro.so', 'build-info.json', 'samba/bin/smbd', 'slotupdate.sh']

class Recovery(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.card = self.root / 'card'
        self.new = self.card / 'System.new'
        self.system = self.card / 'System'
        for directory, value in [(self.new, b'new'), (self.system, b'old')]:
            for name in FILES:
                p = directory / name
                p.parent.mkdir(parents=True, exist_ok=True)
                p.write_bytes(value)
                p.chmod(0o755)
        (self.new / '.manifest').write_text('\n'.join(FILES) + '\n')
        self.keep = ['Config/settings', 'Games/game.gba', 'Saves/game.sav', 'System/custom.txt']
        for name in self.keep:
            p = self.card / name
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_bytes(b'keep')
        self.env = dict(os.environ, SLOT_ROOT=str(self.card), AGS_SPLASH='true')

    def run_installer(self):
        return subprocess.run(['sh', str(INSTALLER)], env=self.env, capture_output=True).returncode

    def unchanged(self):
        for name in FILES:
            self.assertEqual((self.system / name).read_bytes(), b'old')
        self.preserved()

    def preserved(self):
        for name in self.keep:
            self.assertEqual((self.card / name).read_bytes(), b'keep')

    def completed(self):
        self.assertEqual(self.run_installer(), 0)
        self.assertFalse(self.new.exists())
        for name in FILES:
            p = self.system / name
            self.assertEqual(p.read_bytes(), b'new')
            self.assertEqual(p.stat().st_mode & 0o777, 0o755)
        self.preserved()

    def test_copy_failures_and_process_death_at_every_replacement_are_retryable(self):
        # Each iteration uses a fresh card; failures may leave a mixed System,
        # but staging must stay complete and launch must remain gated on retry.
        for fail in ['partial', 'kill']:
            for position in range(1, len(FILES) + 1):
                with self.subTest(fail=fail, position=position):
                    fixture = Recovery(); fixture.setUp()
                    try:
                        bin_dir = fixture.root / 'bin'; bin_dir.mkdir()
                        counter = fixture.root / 'count'
                        script = bin_dir / 'cp'
                        script.write_text('#!/bin/sh\nn=0\n[ ! -f "$COUNT" ] || n=$(cat "$COUNT")\nn=$((n+1))\necho "$n" > "$COUNT"\nif [ "$n" -eq "$FAIL_AT" ]; then\n if [ "$FAIL_KIND" = kill ]; then kill -KILL "$PPID"; exit 1; fi\n for last do :; done\n echo partial > "$last"\n exit 1\nfi\nexec '+shutil.which('cp')+' "$@"\n')
                        script.chmod(0o755)
                        fixture.env.update(PATH=str(bin_dir)+os.pathsep+os.environ['PATH'], COUNT=str(counter), FAIL_AT=str(position), FAIL_KIND=fail)
                        self.assertNotEqual(fixture.run_installer(), 0)
                        self.assertTrue((fixture.new / '.manifest').is_file())
                        for name in FILES:
                            self.assertEqual((fixture.new / name).read_bytes(), b'new')
                        fixture.preserved()
                        fixture.env['PATH'] = os.environ['PATH']
                        fixture.completed()
                    finally:
                        fixture.doCleanups()

    def test_redirected_parents_and_directory_collision_fail_before_any_write(self):
        for where in ['destination', 'staging', 'leaf']:
            with self.subTest(where=where):
                fixture=Recovery(); fixture.setUp()
                try:
                    outside=fixture.root/'outside'; outside.mkdir(); (outside/'sentinel').write_bytes(b'keep')
                    if where=='leaf':
                        p=fixture.system/'gpsp_libretro.so'; p.unlink(); p.mkdir()
                    else:
                        p=(fixture.system if where=='destination' else fixture.new)/'samba'
                        shutil.rmtree(p); p.symlink_to(outside, target_is_directory=True)
                    self.assertNotEqual(fixture.run_installer(),0)
                    self.assertEqual((fixture.system/'slot').read_bytes(),b'old')
                    self.assertEqual(list(outside.iterdir()),[outside/'sentinel'])
                    fixture.preserved()
                finally: fixture.doCleanups()

    def test_redirected_staging_root_is_not_removed(self):
        outside=self.root/'outside'; self.new.rename(outside); self.new.symlink_to(outside,target_is_directory=True)
        self.assertNotEqual(self.run_installer(),0)
        self.assertTrue((outside/'.manifest').exists()); self.unchanged()

    def test_final_sync_failure_keeps_complete_stage_for_retry(self):
        bin_dir=self.root/'bin'; bin_dir.mkdir()
        counter=self.root/'sync-count'
        script=bin_dir/'sync'
        script.write_text('#!/bin/sh\nn=0\n[ ! -f "$COUNT" ] || n=$(cat "$COUNT")\nn=$((n+1))\necho "$n" > "$COUNT"\n[ "$n" -ne 2 ]\n')
        script.chmod(0o755)
        self.env.update(PATH=str(bin_dir)+os.pathsep+os.environ['PATH'],COUNT=str(counter))
        self.assertNotEqual(self.run_installer(),0)
        self.assertTrue((self.new/'.manifest').exists())
        self.preserved()
        self.env['PATH']=os.environ['PATH']; self.completed()

    def test_sync_failure_retains_recovery_source(self):
        bin_dir=self.root/'bin'; bin_dir.mkdir()
        script=bin_dir/'sync'; script.write_text('#!/bin/sh\nexit 1\n'); script.chmod(0o755)
        self.env['PATH']=str(bin_dir)+os.pathsep+os.environ['PATH']
        self.assertNotEqual(self.run_installer(),0); self.unchanged()
        self.assertTrue((self.new/'.manifest').exists())
        self.env['PATH']=os.environ['PATH']; self.completed()

if __name__ == '__main__': unittest.main()
