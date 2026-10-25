//! Bounded robust outline fitting for opaque white-backed label scans.
//! Fits seven numbers to sampled boundary observations, then masks once.

type Shape = [f64; 7];

fn white(rgba: &[u8], i: usize) -> bool {
    let p = &rgba[i * 4..i * 4 + 3];
    let lo = *p.iter().min().unwrap();
    let hi = *p.iter().max().unwrap();
    lo >= 235 && hi - lo <= 20
}

fn distance(p: &Shape, x: f64, y: f64, w: f64, h: f64) -> f64 {
    let u = x - p[5] * (y - h / 2.0);
    let v = y - p[6] * (x - w / 2.0);
    let qx = (u - p[0]).abs() - p[2] + p[4];
    let qy = (v - p[1]).abs() - p[3] + p[4];
    qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - p[4]
}

fn solve(mut a: [[f64; 7]; 7], mut b: Shape) -> Option<Shape> {
    for k in 0..7 {
        let pivot = (k..7).max_by(|&i, &j| a[i][k].abs().total_cmp(&a[j][k].abs()))?;
        if a[pivot][k].abs() < 1e-12 { return None; }
        a.swap(k, pivot);
        b.swap(k, pivot);
        let div = a[k][k];
        for j in k..7 { a[k][j] /= div; }
        b[k] /= div;
        for i in 0..7 {
            if i == k { continue; }
            let f = a[i][k];
            for j in k..7 { a[i][j] -= f * a[k][j]; }
            b[i] -= f * b[k];
        }
    }
    b.iter().all(|v| v.is_finite()).then_some(b)
}

fn fit(points: &[(f64, f64)], w: f64, h: f64) -> Option<Shape> {
    let short = w.min(h);
    let mut p = [w / 2.0, h / 2.0, w / 2.0 - short / 200.0,
        h / 2.0 - short / 100.0, short / 12.0, 0.0, 0.0];
    let lo = [w * 0.48, h * 0.48, w * 0.45, h * 0.45, short / 64.0, -0.03, -0.03];
    let hi = [w * 0.52, h * 0.52, w * 0.51, h * 0.51, short / 5.0, 0.03, 0.03];
    let scale = (short / 440.0).max(0.5);
    let loss = |p: &Shape| points.iter().map(|&(x, y)| {
        let r = distance(p, x, y, w, h) / scale;
        2.0 * scale * scale * ((1.0 + r * r).sqrt() - 1.0)
    }).sum::<f64>();
    let mut damping: f64 = 0.001;
    for _ in 0..60 {
        let mut a = [[0.0; 7]; 7];
        let mut b = [0.0; 7];
        for &(x, y) in points {
            let e = distance(&p, x, y, w, h);
            let weight = 1.0 / (1.0 + (e / scale).powi(2)).sqrt();
            let mut jac = [0.0; 7];
            for k in 0..7 {
                let step = if k < 5 { 0.01 } else { 0.00001 };
                let mut shifted = p;
                shifted[k] += step;
                jac[k] = (distance(&shifted, x, y, w, h) - e) / step;
            }
            for i in 0..7 {
                b[i] -= jac[i] * weight * e;
                for j in 0..7 { a[i][j] += jac[i] * weight * jac[j]; }
            }
        }
        for k in 0..7 { a[k][k] += a[k][k].max(1.0) * damping; }
        let delta = solve(a, b)?;
        let mut candidate = p;
        for k in 0..7 { candidate[k] = (p[k] + delta[k]).clamp(lo[k], hi[k]); }
        if loss(&candidate) < loss(&p) {
            p = candidate;
            damping = (damping / 3.0).max(1e-8);
            if delta[..5].iter().all(|v| v.abs() < 0.0001)
                && delta[5..].iter().all(|v| v.abs() < 0.000001) { break; }
        } else { damping = (damping * 10.0).min(1e8); }
    }
    Some(p)
}

/// Returns changed alpha pixels; every guard runs before any mutation.
pub(crate) fn clear_white_outline(rgba: &mut [u8], w: usize, h: usize) -> usize {
    // Called only for opaque, dimension-checked images by label_mask.
    if [0, w - 1, (h - 1) * w, w * h - 1].iter().any(|&i| !white(rgba, i)) { return 0; }
    let short = w.min(h);
    let stride = (short / 160).max(1);
    let mut points = Vec::new();
    let mut sides = [0; 4];
    for y in (0..h).step_by(stride) {
        if let Some(x) = (0..w).find(|&x| !white(rgba, y * w + x)) {
            if x < short / 6 { points.push((x as f64, y as f64)); sides[0] += 1; }
        }
        if let Some(x) = (0..w).rev().find(|&x| !white(rgba, y * w + x)) {
            if x >= w - short / 6 { points.push((x as f64, y as f64)); sides[1] += 1; }
        }
    }
    for x in (0..w).step_by(stride) {
        if let Some(y) = (0..h).find(|&y| !white(rgba, y * w + x)) {
            if y < short / 6 { points.push((x as f64, y as f64)); sides[2] += 1; }
        }
        if let Some(y) = (0..h).rev().find(|&y| !white(rgba, y * w + x)) {
            if y >= h - short / 6 { points.push((x as f64, y as f64)); sides[3] += 1; }
        }
    }
    // Broad evidence on all sides; blank white labels cannot provide it.
    if sides[0].min(sides[1]) * 2 < h.div_ceil(stride)
        || sides[2].min(sides[3]) * 2 < w.div_ceil(stride) { return 0; }
    let (wf, hf) = (w as f64, h as f64);
    let Some(p) = fit(&points, wf, hf) else { return 0; };
    let close = points.iter().filter(|&&(x, y)| distance(&p, x, y, wf, hf).abs() <= (short as f64 / 100.0).max(1.5)).count();
    if close * 5 < points.len() * 4 { return 0; }
    let mut changed = 0;
    let mut pale = 0;
    let mut exterior = 0;
    for y in 0..h {
        for x in 0..w {
            let d = distance(&p, x as f64, y as f64, wf, hf);
            if d > -1.5 { changed += 1; }
            if d > 0.0 { exterior += 1; pale += usize::from(white(rgba, y * w + x)); }
        }
    }
    // Allow a thin scanned/antialiased edge, never a large crop of printed art.
    if changed == 0 || changed > w * h / 12 || exterior == 0 || pale * 4 < exterior * 3 { return 0; }
    for y in 0..h {
        for x in 0..w {
            let alpha = ((-distance(&p, x as f64, y as f64, wf, hf) - 0.5).clamp(0.0, 1.0) * 255.0).round() as u8;
            rgba[(y * w + x) * 4 + 3] = alpha;
        }
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scan(w: usize, h: usize, tilt: bool) -> (Vec<u8>, Shape) {
        let p = [w as f64 / 2.0, h as f64 / 2.0, w as f64 / 2.0 - 2.0,
            h as f64 / 2.0 - 4.0, h as f64 / 12.0,
            if tilt { 0.003 } else { 0.0 }, if tilt { -0.008 } else { 0.0 }];
        let mut rgba = vec![255; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                if distance(&p, x as f64, y as f64, w as f64, h as f64) <= 0.0 {
                    rgba[(y * w + x) * 4..][..3].copy_from_slice(&[200, 30, 15]);
                }
            }
        }
        (rgba, p)
    }

    #[test]
    fn removes_straight_borders_and_tilted_corners_without_changing_interior() {
        for tilt in [false, true] {
            let (w, h) = (512, 264);
            let (mut rgba, shape) = scan(w, h, tilt);
            // Printed white regions, including a stripe touching the top edge.
            for y in 0..h {
                for x in w / 2..w / 2 + 5 { rgba[(y * w + x) * 4..][..3].fill(255); }
            }
            let ink = ((h / 2) * w + w / 3) * 4;
            rgba[ink..ink + 3].fill(0);
            let before = rgba.clone();
            let changed = clear_white_outline(&mut rgba, w, h);
            assert!(changed > w * 2);
            assert!(changed < w * h / 12);
            assert_eq!(rgba[(h / 2 * w) * 4 + 3], 0);
            assert_eq!(rgba[(w / 3) * 4 + 3], 0);
            assert_eq!(&rgba[ink..ink + 4], &[0, 0, 0, 255]);
            assert_eq!(&rgba[(h / 2 * w + w / 2) * 4..][..4], &[255; 4]);
            for y in 0..h {
                for x in 0..w {
                    let i = (y * w + x) * 4;
                    assert_eq!(&rgba[i..i + 3], &before[i..i + 3]);
                    if distance(&shape, x as f64, y as f64, w as f64, h as f64) < -4.0 {
                        assert_eq!(&rgba[i..i + 4], &before[i..i + 4]);
                    }
                }
            }
            assert!(rgba.chunks_exact(4).any(|p| p[3] > 0 && p[3] < 255));
        }
    }

    #[test]
    fn white_labels_and_unrelated_artwork_are_unchanged() {
        for color in [[255, 255, 255], [242, 241, 240], [64, 64, 64]] {
            let mut rgba = vec![255; 256 * 128 * 4];
            for px in rgba.chunks_exact_mut(4) { px[..3].copy_from_slice(&color); }
            let before = rgba.clone();
            assert_eq!(clear_white_outline(&mut rgba, 256, 128), 0);
            assert_eq!(rgba, before);
        }
        // White frame around an irregular central illustration lacks a nearby
        // rounded die-cut boundary on every side.
        let mut rgba = vec![255; 256 * 128 * 4];
        for y in 40..88 {
            for x in 80..176 { rgba[(y * 256 + x) * 4..][..3].fill(40); }
        }
        let before = rgba.clone();
        assert_eq!(clear_white_outline(&mut rgba, 256, 128), 0);
        assert_eq!(rgba, before);
    }
}
