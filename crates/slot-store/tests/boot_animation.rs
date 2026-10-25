use slot_store::{read_slot_state, write_slot_state, SlotState};

#[test]
fn boot_animation_defaults_on_and_survives_restart_when_disabled() {
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(d.path().join("Config")).unwrap();
    assert!(read_slot_state(d.path()).boot_animation);
    let state = SlotState { boot_animation: false, ..SlotState::default() };
    write_slot_state(d.path(), &state).unwrap();
    assert!(!read_slot_state(d.path()).boot_animation);
    let path = d.path().join("Config/slot.state");
    let old = std::fs::read_to_string(&path).unwrap().lines()
        .filter(|line| !line.starts_with("boot_animation="))
        .map(|line| format!("{line}\n")).collect::<String>();
    std::fs::write(path, old).unwrap();
    assert!(read_slot_state(d.path()).boot_animation);
}
