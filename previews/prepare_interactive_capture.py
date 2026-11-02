"""Add recorder-only access in the disposable CI checkout, never release code."""
from pathlib import Path
import shutil

root = Path(__file__).resolve().parents[1]
for name in ['egl_surface', 'interactive_hero']:
    shutil.copyfile(root / f'previews/{name}.rs', root / f'crates/slot/examples/{name}.rs')
path = root / 'crates/slot/src/frontend.rs'
text = path.read_text()
anchor = '    pub fn app(&self) -> &crate::app::App {'
assert text.count(anchor) == 1
text = text.replace(anchor, '''    pub fn preview_step(&mut self, action:Option<slot_input::Action>) {
        let app=self.session.app_mut();
        if let Some(action)=action {app.apply(action);}
        app.update(1.0/60.0);
    }

''' + anchor)
anchor = '        compositor.draw_list(&self.draws);'
assert text.count(anchor) == 1
# A fixed wallpaper phase makes independently captured actions join seamlessly.
# Cartridge, slot and boot motion all use the unchanged production code.
text = text.replace(anchor, '''        for draw in &mut self.draws {
            if let Draw::Lava{phase,..}=draw {*phase=0.0;}
        }
''' + anchor)
path.write_text(text)
