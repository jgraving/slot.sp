//! Run against the real bundled cores on the ARM64 PR builder.
use slot_retro::{ButtonMask, LibretroCore, RetroCore};
use std::path::{Path, PathBuf};
fn rom() -> PathBuf {
    let mut data = vec![0u8; 0x8000];
    data[..4].copy_from_slice(&0xea00002eu32.to_le_bytes());
    data[0xa0..0xac].copy_from_slice(b"SLOT MEMMAP\0");
    data[0xac..0xb0].copy_from_slice(b"SLMM");
    data[0xb0..0xb2].copy_from_slice(b"00");
    data[0xb2] = 0x96;
    let sum = data[0xa0..0xbd].iter().fold(0u8, |a, b| a.wrapping_add(*b));
    data[0xbd] = 0u8.wrapping_sub(sum).wrapping_sub(0x19);
    data[0xc0..0xc4].copy_from_slice(&0xeafffffeu32.to_le_bytes());
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("achievement-memory.gba");
    std::fs::write(&path, data).unwrap();
    path
}
#[test]
fn both_bundled_gba_cores_expose_complete_work_ram_snapshots() {
    let vendor = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor");
    for name in ["mgba", "gpsp"] {
        let path = vendor.join(format!(
            "{name}_libretro.{}",
            std::env::consts::DLL_EXTENSION
        ));
        if !path.exists() {
            assert!(
                std::env::var_os("SLOT_REQUIRE_CORES").is_none(),
                "missing bundled core: {}",
                path.display()
            );
            continue;
        }
        let mut core = LibretroCore::open(&path).unwrap();
        core.load(&rom()).unwrap();
        for _ in 0..3 {
            core.run_frame(ButtonMask(0));
        }
        let mut ram = vec![0; 0x58000];
        let valid = core.achievement_memory(&mut ram);
        assert_eq!(
            &valid[..2],
            &[0x8000, 0x40000],
            "{name} is missing achievement work RAM"
        );
    }
}
