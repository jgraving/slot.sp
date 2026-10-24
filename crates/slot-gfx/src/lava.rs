//! Bounded, slow background motion; the fragment shader does no trigonometry.
pub const LAVA_PERIOD_MS: u64 = 36_000;

pub fn lava_phase(now_ms: u64) -> f32 {
    (now_ms % LAVA_PERIOD_MS) as f32 * (std::f32::consts::TAU / LAVA_PERIOD_MS as f32)
}

pub fn lava_centres(phase: f32) -> [[f32; 4]; 3] {
    [
        [0.12 + 0.20 * phase.sin(), 0.20 + 0.16 * (phase + 0.8).sin(), 0.55, 0.68],
        [0.88 + 0.22 * (phase + 2.3).sin(), 0.24 + 0.18 * (phase + 3.2).sin(), 0.58, 0.67],
        [0.52 + 0.18 * (phase + 4.0).sin(), -0.08 + 0.15 * (phase + 1.4).sin(), 0.42, 0.58],
    ]
}
