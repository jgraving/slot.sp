use std::path::Path;

use slot_gfx::{SRC_H, SRC_W, WHOLE_TEXTURE};
use slot_store::{Platform, ini};

pub const VIDEO_MODE_FILE: &str = "Config/video_mode.txt";

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum VideoMode {
    #[default]
    Actual,
    Stretch,
    Fit,
    Integer,
}

impl VideoMode {
    pub fn as_str(self) -> &'static str {
        match self {
            VideoMode::Actual => "actual",
            VideoMode::Stretch => "stretch",
            VideoMode::Fit => "fit",
            VideoMode::Integer => "integer",
        }
    }

    pub fn parse(s: &str) -> Option<VideoMode> {
        match s.trim().to_ascii_lowercase().as_str() {
            "actual" => Some(VideoMode::Actual),
            "stretch" => Some(VideoMode::Stretch),
            "fit" => Some(VideoMode::Fit),
            "integer" => Some(VideoMode::Integer),
            _ => None,
        }
    }
}

pub fn video_mode_for(root: &Path, stem: &str) -> VideoMode {
    ini::value(root, VIDEO_MODE_FILE, stem)
        .as_deref()
        .and_then(VideoMode::parse)
        .unwrap_or_default()
}

pub fn write_video_mode(root: &Path, stem: &str, mode: VideoMode) -> std::io::Result<()> {
    ini::write(root, VIDEO_MODE_FILE, stem, mode.as_str())
}

pub fn source_rect(platform: Platform, mode: VideoMode) -> [f32; 4] {
    let (w, h) = platform.picture();
    if mode == VideoMode::Actual || (w, h) == (SRC_W, SRC_H) {
        return WHOLE_TEXTURE;
    }
    if matches!(mode, VideoMode::Fit | VideoMode::Integer) {
        return [
            (SRC_W - w) as f32 / 2.0 / SRC_W as f32,
            (SRC_H - h) as f32 / 2.0 / SRC_H as f32,
            w as f32 / SRC_W as f32,
            h as f32 / SRC_H as f32,
        ];
    }
    let x = SRC_W.saturating_sub(w) / 2;
    [x as f32 / SRC_W as f32, 0.0, w as f32 / SRC_W as f32, 1.0]
}

impl VideoMode {
    pub fn step(self, right: bool) -> Self {
        let modes = [Self::Actual, Self::Stretch, Self::Fit, Self::Integer];
        let at = modes.iter().position(|m| *m == self).unwrap_or(0);
        modes[(at + if right { 1 } else { modes.len() - 1 }) % modes.len()]
    }
    pub fn destination(self, platform: Platform, panel: (u32, u32)) -> Option<[f32; 4]> {
        if !matches!(self, Self::Fit | Self::Integer) {
            return None;
        }
        let (pw, ph) = (panel.0.max(1) as f32, panel.1.max(1) as f32);
        let (w, h) = platform.picture();
        let scale = (pw / w as f32).min(ph / h as f32);
        let scale = if self == Self::Integer && scale >= 1.0 {
            scale.floor()
        } else {
            scale
        };
        let (w, h) = (w as f32 * scale, h as f32 * scale);
        // Place boundaries on physical pixels, then express them in the game's logical canvas.
        Some([
            ((pw - w) / 2.0).floor() / pw * 720.0,
            ((ph - h) / 2.0).floor() / ph * 480.0,
            w / pw * 720.0,
            h / ph * 480.0,
        ])
    }
}
