use slot::kid_policy::{Policy, Rule};
use slot_store::Platform;
const NOW: i64 = 1_800_000_000;
fn ready(rule: Rule) -> (tempfile::TempDir, Policy) {
    let d = tempfile::tempdir().unwrap();
    let mut p = Policy::default();
    p.rule = rule;
    p.minutes = 5;
    p.approved.push((Platform::Gba, "Example".into()));
    std::fs::create_dir_all(Policy::directory(d.path())).unwrap();
    std::fs::write(Policy::directory(d.path()).join("code"), b"test").unwrap();
    (d, p)
}
#[test]
fn daily_budget_survives_restart_and_pauses_do_not_consume_time() {
    let (d, mut p) = ready(Rule::Daily);
    p.begin(NOW).unwrap();
    p.tick(d.path(), NOW, 0, true).unwrap();
    p.tick(d.path(), NOW + 10, 10_000, true).unwrap();
    let before = p.remaining(NOW + 10);
    p.tick(d.path(), NOW + 10, 10_000, false).unwrap();
    p.tick(d.path(), NOW + 110, 110_000, false).unwrap();
    p.tick(d.path(), NOW + 110, 110_000, true).unwrap();
    assert_eq!(p.remaining(NOW + 110), before);
    let mut rebooted = Policy::read(d.path()).unwrap();
    assert_eq!(rebooted.remaining(NOW + 110), 240);
    assert!(rebooted.tick(d.path(), NOW + 110, 0, true).unwrap());
    assert!(rebooted.tick(d.path(), NOW + 170, 60_000, true).unwrap());
    assert!(rebooted.tick(d.path(), NOW + 230, 120_000, true).unwrap());
    assert!(rebooted.tick(d.path(), NOW + 290, 180_000, true).unwrap());
    assert!(!rebooted.tick(d.path(), NOW + 350, 240_000, true).unwrap());
}
#[test]
fn one_off_deadline_counts_sleep_and_cannot_be_refunded_by_reboot() {
    let (d, mut p) = ready(Rule::Session);
    p.begin(NOW).unwrap();
    p.save(d.path()).unwrap();
    let mut p = Policy::read(d.path()).unwrap();
    assert_eq!(p.remaining(NOW + 120), 180);
    assert!(!p.tick(d.path(), NOW + 300, 0, false).unwrap());
}
#[test]
fn bedtime_is_today_and_past_times_are_rejected() {
    let (_, mut p) = ready(Rule::Until);
    let midnight = NOW.div_euclid(86_400) * 86_400;
    p.bedtime = 19 * 60 + 30;
    p.begin(midnight + 19 * 3600).unwrap();
    assert_eq!(p.remaining(midnight + 19 * 3600), 1800);
    assert!(p.begin(midnight + 20 * 3600).is_err());
}
#[test]
fn clock_rollback_blocks_until_a_parent_authorizes_a_new_session() {
    let (d, mut p) = ready(Rule::Session);
    p.begin(NOW).unwrap();
    assert!(!p.tick(d.path(), NOW - 60, 0, true).unwrap());
    let p = Policy::read(d.path()).unwrap();
    assert_eq!(p.remaining(NOW), 0);
}
#[test]
fn failed_budget_write_does_not_grant_play() {
    let (d, mut p) = ready(Rule::Daily);
    p.begin(NOW).unwrap();
    let invalid = d.path().join("file");
    std::fs::write(&invalid, b"not a directory").unwrap();
    assert!(p.tick(&invalid, NOW, 0, true).is_err());
}
#[test]
fn malformed_or_missing_active_policy_is_not_treated_as_unrestricted() {
    let (d, _) = ready(Rule::Daily);
    assert!(Policy::read(d.path()).is_err());
    std::fs::write(Policy::directory(d.path()).join("settings"), b"broken").unwrap();
    assert!(Policy::read(d.path()).is_err());
}

#[test]
fn a_long_frame_cannot_refund_daily_playtime() {
    let (d, mut p) = ready(Rule::Daily);
    p.begin(NOW).unwrap();
    assert!(p.tick(d.path(), NOW, 0, true).unwrap());
    assert!(!p.tick(d.path(), NOW + 300, 300_000, true).unwrap());
    assert_eq!(Policy::read(d.path()).unwrap().remaining(NOW + 300), 0);
}

#[test]
fn warning_milestones_skip_the_start_and_survive_restart() {
    let (d, mut p) = ready(Rule::Session);
    p.minutes = 120;
    p.begin(NOW).unwrap();
    assert_eq!(p.take_warning(d.path(), NOW).unwrap(), None);
    for (elapsed, remaining) in [(3600, 3600), (5400, 1800), (6300, 900), (6900, 300), (7140, 60)] {
        assert_eq!(p.take_warning(d.path(), NOW + elapsed).unwrap(), Some(remaining));
        p = Policy::read(d.path()).unwrap();
        assert_eq!(p.take_warning(d.path(), NOW + elapsed).unwrap(), None);
    }
    assert_eq!(p.take_warning(d.path(), NOW + 7200).unwrap(), None);
}

#[test]
fn short_sessions_skip_immediate_warnings_and_wake_combines_crossed_milestones() {
    let (d, mut p) = ready(Rule::Session);
    p.minutes = 15;
    p.begin(NOW).unwrap();
    assert_eq!(p.take_warning(d.path(), NOW).unwrap(), None);
    assert_eq!(p.take_warning(d.path(), NOW + 600).unwrap(), Some(300));
    p.minutes = 120;
    p.begin(NOW).unwrap();
    // More than one milestone passed, but only one current warning is emitted.
    assert_eq!(p.take_warning(d.path(), NOW + 6600).unwrap(), Some(600));
    assert_eq!(p.take_warning(d.path(), NOW + 6600).unwrap(), None);
    assert_eq!(p.take_warning(d.path(), NOW + 6900).unwrap(), Some(300));
}

#[test]
fn daily_warning_pause_and_idle_time_do_not_use_gameplay_allowance() {
    let (d, mut p) = ready(Rule::Daily);
    p.begin(NOW).unwrap();
    assert!(p.tick(d.path(), NOW, 0, false).unwrap());
    assert!(p.tick(d.path(), NOW + 600, 600_000, false).unwrap());
    assert_eq!(p.remaining(NOW + 600), 300);
    assert!(p.tick(d.path(), NOW + 600, 600_000, true).unwrap());
    assert!(p.tick(d.path(), NOW + 840, 840_000, true).unwrap());
    assert_eq!(p.take_warning(d.path(), NOW + 840).unwrap(), Some(60));
    assert!(p.tick(d.path(), NOW + 840, 840_000, false).unwrap());
    assert!(p.tick(d.path(), NOW + 1440, 1_440_000, false).unwrap());
    assert_eq!(p.remaining(NOW + 1440), 60);
    assert!(p.tick(d.path(), NOW + 1440, 1_440_000, true).unwrap());
    assert_eq!(p.remaining(NOW + 1440), 60);
    assert!(!p.tick(d.path(), NOW + 1500, 1_500_000, true).unwrap());
}

#[test]
fn a_new_session_or_new_day_can_warn_again_but_inactive_mode_cannot() {
    let (d, mut p) = ready(Rule::Session);
    p.minutes = 120;
    p.begin(NOW).unwrap();
    assert_eq!(p.take_warning(d.path(), NOW + 3600).unwrap(), Some(3600));
    p.begin(NOW + 3600).unwrap();
    assert_eq!(p.take_warning(d.path(), NOW + 7200).unwrap(), Some(3600));
    p.leave();
    assert_eq!(p.take_warning(d.path(), NOW + 8700).unwrap(), None);
    p.rule = Rule::Daily;
    p.begin(NOW).unwrap();
    p.tick(d.path(), NOW, 0, true).unwrap();
    p.tick(d.path(), NOW + 3600, 3_600_000, true).unwrap();
    assert_eq!(p.take_warning(d.path(), NOW + 3600).unwrap(), Some(3600));
    p.tick(d.path(), NOW + 86_400, 3_600_000, false).unwrap();
    assert_eq!(p.remaining(NOW + 86_400), 7200);
    assert_eq!(p.take_warning(d.path(), NOW + 86_400).unwrap(), None);
    p.tick(d.path(), NOW + 86_400, 3_600_000, true).unwrap();
    p.tick(d.path(), NOW + 90_000, 7_200_000, true).unwrap();
    assert_eq!(p.take_warning(d.path(), NOW + 90_000).unwrap(), Some(3600));
}

#[test]
fn bedtime_warning_keeps_its_original_meter_budget_across_restart() {
    let (d, mut p) = ready(Rule::Until);
    let start = NOW.div_euclid(86_400) * 86_400 + 18 * 3600;
    p.bedtime = 20 * 60;
    p.begin(start).unwrap();
    assert_eq!(p.time_budget(), 7200);
    assert_eq!(p.take_warning(d.path(), start + 6300).unwrap(), Some(900));
    let mut p = Policy::read(d.path()).unwrap();
    assert_eq!(p.time_budget(), 7200);
    assert_eq!(p.take_warning(d.path(), start + 6300).unwrap(), None);
}

#[test]
fn older_policy_formats_upgrade_and_invalid_warning_data_fails_closed() {
    let (d, mut p) = ready(Rule::Session);
    p.minutes = 120;
    p.begin(NOW).unwrap();
    p.save(d.path()).unwrap();
    let file = Policy::directory(d.path()).join("settings");
    let saved = std::fs::read_to_string(&file).unwrap();
    for n in [9, 10] {
        let mut lines: Vec<_> = saved.lines().map(str::to_owned).collect();
        lines[1] = lines[1].split_whitespace().take(n).collect::<Vec<_>>().join(" ");
        std::fs::write(&file, lines.join("\n") + "\n").unwrap();
        let mut old = Policy::read(d.path()).unwrap();
        assert_eq!(old.time_budget(), 7200);
        assert_eq!(old.take_warning(d.path(), NOW + 3600).unwrap(), Some(3600));
    }
    let bad = saved.replacen(" 0 7200\n", " 32 7200\n", 1);
    assert_ne!(saved, bad);
    std::fs::write(&file, bad).unwrap();
    assert!(Policy::read(d.path()).is_err());
}

#[test]
fn parental_volume_caps_cover_the_full_device_range_and_survive_restart(){
    let (d,mut p)=ready(Rule::Daily);for volume in [0u8,10,80,90,100]{p.volume_limit=volume;p.save(d.path()).unwrap();assert_eq!(Policy::read(d.path()).unwrap().volume_limit,volume);}
}
