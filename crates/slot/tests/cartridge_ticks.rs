mod common;
use slot::{app::App, audio::Sfx};
use slot_input::{Action, Btn};

#[test]
fn carousel_clips_are_short_quiet_stereo_and_distinct() {
    let sounds = [Sfx::Cartridge(0), Sfx::Cartridge(1), Sfx::Cartridge(2), Sfx::Cartridge(3), Sfx::Cartridge(4)];
    let mut clips = Vec::new();
    for sound in sounds {
        for rate in [32_768, 44_100, 48_000] {
            let clip = sound.render(rate);
            assert!(!clip.is_empty());
            assert!(clip.len() < rate as usize / 4);
            assert!(clip.chunks_exact(2).all(|p| p[0] == p[1]));
            let peak = clip.iter().map(|v| v.unsigned_abs()).max().unwrap();
            assert!((100..6000).contains(&peak), "{sound:?}: {peak}");
            assert!(clip[0].abs() < 10);
            assert!(clip.last().unwrap().abs() < 10);
        }
        clips.push(sound.render(48_000));
    }
    for i in 0..clips.len() {
        for j in i + 1..clips.len() { assert_ne!(clips[i], clips[j]); }
    }
    assert_eq!(Sfx::Cartridge(5).render(48_000), clips[0]);
    assert!(Sfx::Cartridge(0).render(0).is_empty());
    if let Ok(dir) = std::env::var("SLOT_TACTILE_PREVIEW") {
        std::fs::create_dir_all(&dir).unwrap();
        for (name, clip) in ["cartridge", "cartridge-variant"].into_iter().zip(clips) {
            let bytes: Vec<_> = clip.into_iter().flat_map(i16::to_le_bytes).collect();
            std::fs::write(
                std::path::Path::new(&dir).join(format!("{name}.pcm")),
                bytes,
            )
            .unwrap();
        }
    }
}

#[test]
fn motion_emits_one_tick_and_idle_is_silent() {
    let d = common::tmp_root_with_carts(&["Alpha", "Beta", "Gamma"]);
    common::clocked(d.path());
    let mut a = App::boot(d.path());
    a.update(0.1);
    assert!(a.take_sfx().is_none());
    a.apply(Action::GbaDown(Btn::Right));
    a.apply(Action::GbaUp(Btn::Right));
    let mut heard = vec![];
    for _ in 0..60 {
        a.update(0.016);
        if let Some(sound) = a.take_sfx() {
            heard.push(sound);
        }
    }
    assert_eq!(heard.len(), 1);
    assert!(matches!(heard[0], Sfx::Cartridge(_)));
}

#[test]
fn continuous_carouselling_rotates_all_five_without_repeats_across_counter_wrap() {
    let d = common::tmp_root_with_carts(&["Alpha", "Beta", "Gamma"]);
    common::clocked(d.path());
    let mut a = App::boot(d.path());
    a.update(1.0);
    let mut heard = Vec::new();
    for _ in 0..270 {
        a.apply(Action::GbaDown(Btn::Right));
        a.apply(Action::GbaUp(Btn::Right));
        for _ in 0..35 {
            a.update(0.016);
            if let Some(Sfx::Cartridge(v)) = a.take_sfx() { heard.push(v); }
        }
    }
    assert!(heard.len() > 256);
    assert!(heard.windows(2).all(|p| p[0] != p[1]));
    for variant in 0..5 { assert!(heard.contains(&variant)); }
}
