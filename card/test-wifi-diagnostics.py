#!/usr/bin/env python3
"""Run the real helper's read-only diagnostic action against an isolated OS fixture."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parent / 'System/slotwifi.sh'

class Diagnostics(unittest.TestCase):
    def test_allowlisted_facts_without_running_network_commands_or_reading_secrets(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            net, run, binaries = (root / x for x in ('net', 'run', 'bin'))
            for path in (net / 'wlan0', run, binaries): path.mkdir(parents=True)
            (run / 'slotwifi.owner').write_text('owner')
            (run / 'slotwifi-watch.pid').write_text(str(os.getpid()))
            (run / 'slotwifi-udhcpc.pid').write_text('INVALID_SECRET_PID')
            (run / 'slotwifi.conf').write_text('psk=SECRET_PASSWORD\nssid=SECRET_SSID\n')
            resolver = root / 'resolv.conf'
            resolver.write_text('search PRIVATE_DOMAIN\nnameserver 192.0.2.53\n')
            trap = root / 'commands-ran'
            env = dict(os.environ, AGS_NET_SYS=str(net), AGS_RUN=str(run), AGS_RESOLV_CONF=str(resolver))
            for variable in ('AGS_WPA_CLI', 'AGS_WPA_SUPPLICANT', 'AGS_UDHCPC', 'AGS_IP', 'AGS_RFKILL'):
                fake = binaries / variable
                fake.write_text(f'#!/bin/sh\necho unexpected >> "{trap}"\nexit 99\n')
                fake.chmod(0o755)
                env[variable] = str(fake)
            result = subprocess.run(['/bin/sh', str(SCRIPT), 'diagnostics'], env=env, capture_output=True, text=True, check=True, timeout=3)
            facts = dict(line.split('=', 1) for line in result.stdout.splitlines())
            self.assertEqual(facts['interface_present'], 'yes')
            self.assertEqual(facts['watcher'], 'alive')
            self.assertEqual(facts['dhcp_client'], 'stale')
            self.assertEqual(facts['resolver_configured'], 'yes')
            self.assertEqual(facts['link_session'], 'no')
            self.assertFalse(trap.exists(), 'diagnostics must not invoke radio/DHCP mutations')
            self.assertFalse(result.stderr)
            for secret in ('SECRET', 'PRIVATE', '192.0.2.', str(root)):
                self.assertNotIn(secret, result.stdout)
            for value in facts.values(): self.assertIn(value, ('yes', 'no', 'alive', 'stale', 'missing'))
            self.assertEqual((run / 'slotwifi.conf').read_text(), 'psk=SECRET_PASSWORD\nssid=SECRET_SSID\n')
            (run / 'slotwifi-watch.pid').unlink()
            env['AGS_UDHCPC'] = str(root / 'missing-command')
            second = subprocess.check_output(['/bin/sh', str(SCRIPT), 'diagnostics'], env=env, text=True)
            self.assertIn('watcher=missing\n', second)
            self.assertIn('udhcpc_available=no\n', second)

if __name__ == '__main__': unittest.main()
