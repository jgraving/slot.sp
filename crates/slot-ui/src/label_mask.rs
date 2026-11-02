//! Conservative cleanup of opaque black or white scanner backgrounds around rounded labels.
//! Runs on source pixels once, before resizing; the device caches the result.

/// Returns the number of pixels made transparent. Ambiguous images stay unchanged.
pub(crate) fn clear_corner_background(rgba: &mut [u8], w: u32, h: u32) -> usize {
    let (w, h) = (w as usize, h as usize);
    let n = w * h;
    if w.min(h) < 32 || rgba.len() != n * 4 || rgba.chunks_exact(4).any(|p| p[3] != 255) {
        return 0; // Respect existing alpha, including artist-supplied masks.
    }
    // White print can touch the exterior: validate a shape instead of flooding.
    // Keep the established black-background detector separate.
    let white = rgba[..3].iter().all(|&c| c >= 247);
    if white {
        let changed = crate::label_fit::clear_white_outline(rgba, w, h);
        return if changed == 0 { clear_white_template(rgba, w, h) } else { changed };
    }
    let background = |i: usize| {
        rgba[i * 4..i * 4 + 3]
            .iter()
            .all(|&c| c <= 8)
    };
    let corners = [0, w - 1, (h - 1) * w, n - 1];
    if corners.iter().any(|&i| !background(i)) {
        return 0;
    }
    // A few isolated background corner pixels or a one-pixel frame are not a scan cutout.
    let diagonal = (w.min(h) / 200).max(2);
    let diagonals = [
        diagonal * w + diagonal,
        diagonal * w + w - 1 - diagonal,
        (h - 1 - diagonal) * w + diagonal,
        (h - 1 - diagonal) * w + w - 1 - diagonal,
    ];
    if diagonals.iter().any(|&i| !background(i)) {
        return 0;
    }
    let rim = w.min(h) / 8;
    let budget = n / 12;
    let mut seen = vec![false; n];
    let mut queue = Vec::new();
    for i in corners {
        seen[i] = true;
        queue.push(i);
    }
    let mut at = 0;
    while at < queue.len() {
        if queue.len() > budget {
            return 0;
        }
        let i = queue[at];
        let (x, y) = (i % w, i / w);
        if x >= rim && x < w - rim && y >= rim && y < h - rim {
            return 0; // Background flowing into the artwork is not a safe background mask.
        }
        at += 1;
        for next in [
            (x > 0).then(|| i - 1),
            (x + 1 < w).then_some(i + 1),
            (y > 0).then(|| i - w),
            (y + 1 < h).then_some(i + w),
        ]
        .into_iter()
        .flatten()
        {
            if !seen[next] && background(next) {
                seen[next] = true;
                queue.push(next);
            }
        }
    }
    if diagonals.iter().any(|&i| !seen[i]) {
        return 0;
    }
    // Do not change any bytes until the complete candidate has passed every guard.
    for &i in &queue {
        rgba[i * 4 + 3] = 0;
    }
    queue.len()
}

/// White artwork can touch the scan background. Fit a conservative outline
/// instead of following white pixels into the label. Never erase nonwhite pixels.
fn clear_white_template(rgba: &mut [u8], w: usize, h: usize) -> usize {
    let white = |i: usize| rgba[i * 4..i * 4 + 3].iter().all(|&c| c >= 247);
    if [0, w - 1, (h - 1) * w, w * h - 1].iter().any(|&i| !white(i)) {
        return 0;
    }
    let short = w.min(h);
    let stride = (short / 160).max(1);
    let band = (short / 40).max(2) as f64;
    let mut best = None;
    for radius in ((short / 24).max(3)..=short / 5).step_by((short / 320).max(1)) {
        let r = radius as f64;
        let mut outside_count = 0;
        let mut outside_ink = 0;
        let mut band_count = 0;
        let mut band_ink = 0;
        for y in (0..h).step_by(stride) {
            for x in (0..w).step_by(stride) {
                let dx = x.min(w - 1 - x) as f64 + 0.5;
                let dy = y.min(h - 1 - y) as f64 + 0.5;
                let outside = outside_round(dx, dy, r);
                if outside {
                    outside_count += 1;
                    outside_ink += usize::from(!white(y * w + x));
                } else if outside_round(dx - band, dy - band, (r - band).max(0.0)) {
                    band_count += 1;
                    band_ink += usize::from(!white(y * w + x));
                }
            }
        }
        // At least 99.5% white exterior plus substantial colour immediately
        // inside the outline. A plain white label cannot supply this evidence.
        if outside_count >= 4 && outside_ink * 200 <= outside_count
            && band_count > 0 && band_ink * 2 >= band_count {
            best = Some(radius);
        }
    }
    let Some(radius) = best else { return 0; };
    // Verify exterior coverage again at full resolution before changing bytes.
    let mut pixels = Vec::new();
    let mut ink = 0;
    for y in 0..h {
        for x in 0..w {
            if outside_round(x.min(w - 1 - x) as f64 + 0.5,
                y.min(h - 1 - y) as f64 + 0.5, radius as f64) {
                if white(y * w + x) { pixels.push(y * w + x); } else { ink += 1; }
            }
        }
    }
    if ink * 200 > pixels.len() + ink { return 0; }
    for &i in &pixels { rgba[i * 4 + 3] = 0; }
    pixels.len()
}

fn outside_round(dx: f64, dy: f64, radius: f64) -> bool {
    dx < 0.0 || dy < 0.0 || (dx < radius && dy < radius
        && (radius - dx).powi(2) + (radius - dy).powi(2) > radius * radius)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scan() -> Vec<u8> {
        let mut rgba = vec![255; 128 * 64 * 4];
        for y in 0usize..64 {
            for x in 0usize..128 {
                if x.min(127 - x) + y.min(63 - y) < 7 {
                    rgba[(y * 128 + x) * 4..][..3].fill(0);
                }
            }
        }
        rgba
    }

    fn white_scan(level: u8) -> Vec<u8> {
        let mut rgba = vec![255; 128 * 64 * 4];
        for y in 0usize..64 {
            for x in 0usize..128 {
                let dx = x.min(127 - x) as f64 + 0.5;
                let dy = y.min(63 - y) as f64 + 0.5;
                let outside = dx < 12.0 && dy < 12.0
                    && (12.0 - dx).powi(2) + (12.0 - dy).powi(2) > 144.0;
                rgba[(y * 128 + x) * 4..][..3].fill(if outside { level } else { 80 });
            }
        }
        rgba
    }

    #[test]
    fn clears_only_corner_connected_scanner_black_and_preserves_printed_black() {
        let mut rgba = scan();
        let ink = (30 * 128 + 60) * 4;
        rgba[ink..ink + 3].fill(0);
        let before = rgba.clone();
        assert_eq!(clear_corner_background(&mut rgba, 128, 64), 112);
        assert_eq!(&rgba[ink..ink + 4], &[0, 0, 0, 255]);
        for (old, new) in before.chunks_exact(4).zip(rgba.chunks_exact(4)) {
            assert_eq!(&old[..3], &new[..3]);
            if new[3] != 0 {
                assert_eq!(old, new);
            }
        }
    }

    #[test]
    fn white_scan_backgrounds_preserve_white_print_and_black_ink() {
        for level in [247, 255] {
            let mut rgba = white_scan(level);
            for px in rgba.chunks_exact_mut(4) {
                // Keep the fitted exterior and interior unchanged.
                px[3] = 255;
            }
            let print = (30 * 128 + 60) * 4;
            rgba[print..print + 3].fill(255);
            let ink = print + 4;
            rgba[ink..ink + 3].fill(0);
            let before = rgba.clone();
            assert!(clear_corner_background(&mut rgba, 128, 64) > 0);
            for (old, new) in before.chunks_exact(4).zip(rgba.chunks_exact(4)) {
                assert_eq!(&old[..3], &new[..3]);
                if new[3] != 0 {
                    assert_eq!(old, new);
                }
            }
            assert_eq!(rgba[print + 3], 255);
            assert_eq!(rgba[ink + 3], 255);
        }
    }

    #[test]
    fn ambiguous_white_artwork_and_mixed_corners_are_unchanged() {
        let mut candidates = vec![vec![255; 128 * 64 * 4]];
        let mut spill = scan();
        for px in spill.chunks_exact_mut(4) {
            let color = if px[0] == 0 { 255 } else { 80 };
            px[..3].fill(color);
        }
        for t in 0..32 {
            spill[(t * 128 + t) * 4..][..3].fill(255);
            spill[(t * 128 + t + 1) * 4..][..3].fill(255);
        }
        let before = spill.clone();
        clear_corner_background(&mut spill, 128, 64);
        for y in 16..48 {
            for x in 16..112 {
                let i = (y * 128 + x) * 4;
                assert_eq!(&spill[i..i + 4], &before[i..i + 4]);
            }
        }
        let mut mixed = scan();
        mixed[..3].fill(255);
        candidates.push(mixed);
        for mut rgba in candidates {
            let before = rgba.clone();
            assert_eq!(clear_corner_background(&mut rgba, 128, 64), 0);
            assert_eq!(rgba, before);
        }
    }

    #[test]
    fn ambiguous_black_artwork_is_left_byte_for_byte_unchanged() {
        let mut dark = vec![0; 128 * 64 * 4];
        for px in dark.chunks_exact_mut(4) {
            px[3] = 255;
        }
        let before = dark.clone();
        assert_eq!(clear_corner_background(&mut dark, 128, 64), 0);
        assert_eq!(dark, before);
        let mut spill = scan();
        for t in 0..32 {
            spill[(t * 128 + t) * 4..][..3].fill(0);
        }
        // Four-connected bridge to black artwork in the central protected area.
        for t in 0..31 {
            spill[(t * 128 + t + 1) * 4..][..3].fill(0);
        }
        let before = spill.clone();
        assert_eq!(clear_corner_background(&mut spill, 128, 64), 0);
        assert_eq!(spill, before);
    }

    #[test]
    fn existing_transparency_and_square_art_are_not_reinterpreted() {
        let mut transparent = scan();
        transparent[3] = 0;
        let before = transparent.clone();
        assert_eq!(clear_corner_background(&mut transparent, 128, 64), 0);
        assert_eq!(transparent, before);
        let mut square = vec![255; 128 * 64 * 4];
        for i in [0, 127, 63 * 128, 128 * 64 - 1] {
            square[i * 4..i * 4 + 3].fill(0);
        }
        let before = square.clone();
        assert_eq!(clear_corner_background(&mut square, 128, 64), 0);
        assert_eq!(square, before);
    }
}

/// Round only fully opaque rectangular output labels; authored alpha is preserved.
pub(crate) fn round_sharp_corners(rgba:&mut[u8],w:u32,h:u32)->bool {
    if w<16 || h<16 || rgba.len()!=(w*h*4)as usize {return false;}
    let corners=[(0,0),(w-1,0),(0,h-1),(w-1,h-1)];
    for &(cx,cy) in &corners {for dy in 0..2 {for dx in 0..2 {
        let x=if cx==0 {dx}else{cx-dx};let y=if cy==0 {dy}else{cy-dy};
        if rgba[((y*w+x)*4+3)as usize]<248 {return false;}
    }}}
    let radius=(w.min(h)as f32*0.055).clamp(1.5,12.0);
    let edge=radius.ceil()as u32;
    for y in 0..edge {for x in 0..edge {
        let mut inside=0u32;
        for sy in 0..4 {for sx in 0..4 {
            let dx=radius-(x as f32+(sx as f32+0.5)/4.0);
            let dy=radius-(y as f32+(sy as f32+0.5)/4.0);
            if dx*dx+dy*dy<=radius*radius {inside+=1;}
        }}
        for (xx,yy) in [(x,y),(w-1-x,y),(x,h-1-y),(w-1-x,h-1-y)] {
            let i=((yy*w+xx)*4+3)as usize;rgba[i]=(u32::from(rgba[i])*inside/16)as u8;
        }
    }}
    true
}
#[cfg(test)]mod sharp_corner_tests {
    use super::*;
    #[test]fn rectangles_gain_small_antialiased_corners_without_changing_rgb_or_interior() {
        let (w,h)=(240,120);let mut art=vec![255;(w*h*4)as usize];
        for (i,p) in art.chunks_exact_mut(4).enumerate(){p[0]=(i%251)as u8;p[1]=70;p[2]=125;}
        let before=art.clone();assert!(round_sharp_corners(&mut art,w,h));
        assert_eq!(art[3],0);assert!(art.chunks_exact(4).any(|p|p[3]>0&&p[3]<255));
        for (i,(a,b)) in art.chunks_exact(4).zip(before.chunks_exact(4)).enumerate(){
            assert_eq!(&a[..3],&b[..3]);let(x,y)=(i as u32%w,i as u32/w);
            if x>12 && x<w-12 || y>12 && y<h-12 {assert_eq!(a,b);}
        }
        let rounded=art.clone();assert!(!round_sharp_corners(&mut art,w,h));assert_eq!(art,rounded);
    }
    #[test]fn authored_transparency_and_tiny_images_are_unchanged() {
        let mut alpha=vec![255;64*32*4];alpha[3]=128;let before=alpha.clone();
        assert!(!round_sharp_corners(&mut alpha,64,32));assert_eq!(alpha,before);
        let mut tiny=vec![255;8*8*4];assert!(!round_sharp_corners(&mut tiny,8,8));
    }
}
