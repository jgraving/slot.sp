//! Analytic convection orbits and area-preserving blob deformation.
//! This is a lightweight fluid-inspired model, not a Navier–Stokes solver.
pub const LAVA_PERIOD_MS: u64 = 120_000;

pub fn lava_phase(now_ms: u64) -> f32 {
    (now_ms % LAVA_PERIOD_MS) as f32 * (std::f32::consts::TAU / LAVA_PERIOD_MS as f32)
}

pub fn lava_centres(phase: f32) -> [[f32; 4]; 6] {
    std::array::from_fn(|i| {
        let k = i as f32;
        let t = phase * (if i % 2 == 0 { 1.0 } else { 2.0 }) + k * 1.0472;
        // Counter-rotating circulation: warm wax rises, cool wax returns down the sides.
        let lane = [0.25, 0.70, 0.48, 0.22, 0.76, 0.51][i];
        let x = lane + 0.12 * t.sin() + 0.025 * (3.0 * t + k).sin();
        let y = 0.32 + 0.28 * t.cos();
        // Stretch along the direction of travel, conserving the ellipse's area.
        let stretch = 1.0 + 0.18 * (2.0 * t + 0.6).sin();
        let radius = [0.30, 0.27, 0.32, 0.22, 0.24, 0.20][i];
        [x, y, radius / stretch, radius * 1.20 * stretch]
    })
}
