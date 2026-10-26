use slot_store::{Core, Platform, ini};
use std::path::Path;
const FILE: &str = "Config/frame-blending.txt";
fn key(p: Platform) -> &'static str {
    match p {
        Platform::Gba => "GBA",
        Platform::Gb => "GB",
        Platform::Gbc => "GBC",
    }
}
pub fn read(root: &Path, platform: Platform) -> bool {
    ini::value(root, FILE, key(platform)).as_deref() == Some("mix")
}
pub fn write(root: &Path, platform: Platform, on: bool) -> std::io::Result<()> {
    ini::write(root, FILE, key(platform), if on { "mix" } else { "off" })
}
pub fn option(core: Core, on: bool) -> (&'static str, &'static str) {
    match core {
        Core::Mgba => ("mgba_interframe_blending", if on { "mix" } else { "OFF" }),
        Core::Gpsp => ("gpsp_frame_mixing", if on { "enabled" } else { "disabled" }),
    }
}
