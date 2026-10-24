//! Synthetic carousel-only fixture for the existing headless GL recorder.
use slot_store::{write_slot_state, Platform, Recent, SlotState};
fn main() {
    let root = std::path::PathBuf::from(std::env::args().nth(1).expect("fixture root"));
    slot::root::ensure(&root);
    let names = ["Emerald", "Fusion", "Ruby", "Sapphire", "WarioWare"];
    for name in names {
        let mut header = vec![0u8; 256];
        let title = name.to_uppercase();
        header[0xa0..0xa0 + title.len()].copy_from_slice(title.as_bytes());
        std::fs::write(root.join("Games/GBA").join(format!("{name}.gba")), header).unwrap();
    }
    std::fs::write(root.join(slot_store::CART_SHELL_FILE),
        "Emerald = auto 239c47 clear\nRuby = auto b5253b clear\nSapphire = auto 2854b5 clear\n"
    ).unwrap();
    let carts = slot_store::scan(&root).unwrap();
    for name in ["Emerald", "Ruby", "Sapphire"] {
        let cart = carts.iter().find(|cart| cart.stem == name).unwrap();
        assert_eq!(cart.shell.unwrap().finish, slot_store::ShellFinish::Clear);
    }
    write_slot_state(&root, &SlotState {
        clock_set: true,
        shelf: Some("gba".into()),
        shelf_cart: Some(Recent { platform: Platform::Gba, stem: "Emerald".into() }),
        favorites: names.into_iter().map(|name| Recent { platform: Platform::Gba, stem: name.into() }).collect(),
        shell_colour: slot_store::ShellColour::Graphite,
        wallpaper: slot_store::WallpaperPick::File("twilight.png".into()),
        ..SlotState::default()
    }).unwrap();
}
