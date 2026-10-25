use slot_store::{read_slot_state, write_slot_state, SlotState};
#[test]
fn carousel_sound_preference_migrates_and_persists() {
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(d.path().join("Config")).unwrap();
    assert!(read_slot_state(d.path()).carousel_sounds);
    let s = SlotState { carousel_sounds: false, ..SlotState::default() };
    write_slot_state(d.path(), &s).unwrap();
    assert!(!read_slot_state(d.path()).carousel_sounds);
    let p = d.path().join("Config/slot.state");
    let old = std::fs::read_to_string(&p).unwrap().lines()
        .filter(|l| !l.starts_with("carousel_sounds="))
        .map(|l| format!("{l}\n")).collect::<String>();
    std::fs::write(p, old).unwrap();
    assert!(read_slot_state(d.path()).carousel_sounds);
}
