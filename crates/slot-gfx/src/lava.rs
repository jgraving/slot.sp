//! Smooth drifting pools with an eight-second breathing pulse, computed once per frame.
pub const LAVA_PERIOD_MS: u64 = 24_000;

pub fn lava_phase(now_ms: u64) -> f32 {
    (now_ms % LAVA_PERIOD_MS) as f32 * (std::f32::consts::TAU / LAVA_PERIOD_MS as f32)
}

pub fn lava_centres(phase: f32) -> [[f32; 4]; 3] {
    let breath = |offset: f32| 1.0 + 0.22 * (phase * 3.0 + offset).sin();
    [
        [0.12 + 0.30 * phase.sin(), 0.20 + 0.24 * (phase + 0.8).sin(), 0.60 * breath(0.0), 0.72 * breath(0.0)],
        [0.88 + 0.32 * (phase + 2.3).sin(), 0.24 + 0.26 * (phase + 3.2).sin(), 0.62 * breath(1.2), 0.72 * breath(1.2)],
        [0.52 + 0.26 * (phase + 4.0).sin(), -0.08 + 0.22 * (phase + 1.4).sin(), 0.54 * breath(2.4), 0.66 * breath(2.4)],
    ]
}
