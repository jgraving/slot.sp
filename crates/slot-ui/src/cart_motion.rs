//! Cartridge depth over normalized travel time. The renderer uses this depth
//! directly, so the contacts, plastic and sound share the same clock.

/// A steady hand pushes most of the way, then the connector snaps home.
pub fn insert_depth(progress: f32) -> f32 {
    let u = progress.clamp(0.0, 1.0);
    if u == 1.0 { return 1.0; }
    const SNAP: f32 = 0.84;
    const DEPTH: f32 = 0.86;
    const SPEED: f32 = 0.22;
    if u < SNAP {
        let t = u / SNAP;
        let t2 = t * t;
        let t3 = t2 * t;
        DEPTH * (3.0 * t2 - 2.0 * t3) + SPEED * SNAP * (t3 - t2)
    } else {
        let t = u - SNAP;
        let duration = 1.0 - SNAP;
        let acceleration = (1.0 - DEPTH - SPEED * duration) / (duration * duration);
        (DEPTH + SPEED * t + acceleration * t * t).min(1.0)
    }
}

/// A short release pops the cartridge up, still in the slot, before the hand
/// draws it out and comes gently to rest on the carousel.
pub fn eject_depth(progress: f32) -> f32 {
    let u = progress.clamp(0.0, 1.0);
    const RELEASE: f32 = 0.20;
    const HELD: f32 = 0.82;
    if u < RELEASE {
        HELD + (1.0 - HELD) * (1.0 - u / RELEASE).powi(3)
    } else {
        (HELD * (1.0 - crate::slot_chrome::ease((u - RELEASE) / (1.0 - RELEASE)))).clamp(0.0, 1.0)
    }
}

/// Preserve the current pose when a load fails or an insertion is cancelled.
pub fn eject_progress_for_depth(depth: f32) -> f32 {
    let depth = depth.clamp(0.0, 1.0);
    if depth >= 1.0 { return 0.0; }
    if depth <= 0.0 { return 1.0; }
    let (mut low, mut high) = (0.0, 1.0);
    for _ in 0..24 {
        let mid = (low + high) * 0.5;
        if eject_depth(mid) > depth { low = mid; } else { high = mid; }
    }
    (low + high) * 0.5
}
