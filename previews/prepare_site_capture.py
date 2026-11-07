"""Prepare recorder-only mutable fixture access in a disposable checkout."""
from pathlib import Path
import shutil

root = Path(__file__).resolve().parents[1]
for name in ('egl_surface', 'site_refresh'):
    shutil.copyfile(root/f'previews/{name}.rs', root/f'crates/slot/examples/{name}.rs')
path = root/'crates/slot/src/frontend.rs'
text = path.read_text()
marker = '    pub fn app(&self) -> &crate::app::App {'
assert text.count(marker) == 1
assert 'pub fn app_mut(' not in text
path.write_text(text.replace(marker,
    '    pub fn app_mut(&mut self) -> &mut crate::app::App { self.session.app_mut() }\n\n'+marker))
