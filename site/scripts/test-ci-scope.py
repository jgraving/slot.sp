#!/usr/bin/env python3
"""Regression contracts for OS, guide, model and deployment routing."""
import importlib.util
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('scope', ROOT / 'site/scripts/ci-scope.py')
scope = importlib.util.module_from_spec(spec)
spec.loader.exec_module(scope)


class WebsiteRouting(unittest.TestCase):
    def test_device_and_recovery_changes_do_not_start_website_checks(self):
        for paths in [[], ['crates/slot/src/app.rs'], ['card/System/slotwifi.sh'],
                      ['WORKLOG.md', 'CURRENT_WORK.md', 'AGENTS.md'], ['README.md'],
                      ['crates/slot/src/network.rs', 'WORKLOG.md']]:
            with self.subTest(paths=paths):
                self.assertEqual(scope.classify(paths), (False, False))

    def test_published_guides_keep_build_checks_without_3d(self):
        for guide in scope.guides():
            self.assertEqual(scope.classify([guide]), (True, False), guide)
        for path in ['site/docs/questions.md', 'previews/current/provenance.json',
                     'tools/check-site-navigation.cjs', 'site/studio.json',
                     'site/docs/assets/launcher/cart-studio-dark.webp',
                     'site/overrides/partials/footer.html']:
            self.assertEqual(scope.classify([path]), (True, False), path)

    def test_homepage_model_and_shared_dependencies_require_3d(self):
        for path in ['site/docs/index.md', 'site/docs/assets/sp-model.js',
                     'site/docs/assets/slot.css', 'site/docs/assets/hero-player.js',
                     'site/docs/assets/vendor/three-r180.min.js', 'site/overrides/home.html',
                     'site/overrides/main.html', 'site/zensical.toml',
                     'site/scripts/check-sp-video.cjs', 'previews/sp-model/model.glb',
                     'previews/interactive/provenance.json', 'tools/prepare-site.py',
                     '.github/workflows/pages.yml', 'site/scripts/ci-scope.py']:
            self.assertEqual(scope.classify([path]), (True, True), path)

    def test_manual_dispatch_checks_everything(self):
        self.assertEqual(scope.classify([], force=True), (True, True))

    def test_workflow_only_triggers_for_actual_root_guides(self):
        workflow = (ROOT / '.github/workflows/pages.yml').read_text()
        self.assertNotIn("'*.md'", workflow)
        for guide in scope.guides():
            self.assertEqual(workflow.count("- '" + guide + "'"), 2, guide)
        for path in ['WORKLOG.md', 'CURRENT_WORK.md', 'AGENTS.md']:
            self.assertNotIn("- '" + path + "'", workflow)

    def test_deploy_accepts_intentionally_skipped_model_but_not_failure(self):
        workflow = (ROOT / '.github/workflows/pages.yml').read_text()
        deploy = workflow.split('  deploy:', 1)[1]
        self.assertIn('always()', deploy)
        self.assertIn("needs.build.result == 'success'", deploy)
        self.assertIn("needs.model.result == 'success' || (needs.changes.outputs.model == 'false' && needs.model.result == 'skipped')", deploy)
        self.assertIn('needs: [changes, build, model]', deploy)


if __name__ == '__main__':
    unittest.main()
