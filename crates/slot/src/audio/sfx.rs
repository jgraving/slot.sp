const INSERT: &[u8] = include_bytes!("../../assets/insert.pcm");
const POWER_SWITCH: &[u8] = include_bytes!("../../assets/power-switch.pcm");
const EJECT: &[u8] = include_bytes!("../../assets/eject.pcm");
const CAROUSEL: [&[u8]; 5] = [
    include_bytes!("../../assets/carousel-0.pcm"),
    include_bytes!("../../assets/carousel-1.pcm"),
    include_bytes!("../../assets/carousel-2.pcm"),
    include_bytes!("../../assets/carousel-3.pcm"),
    include_bytes!("../../assets/carousel-4.pcm"),
];
const ASSET_HZ: f32 = 48_000.0;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Sfx {
    PowerSwitch,
    Insert,
    Eject,
    Cartridge(u8),
}

impl Sfx {
    pub fn lead(self) -> f32 {
        match self {
            Sfx::PowerSwitch => 0.0,
            Sfx::Insert => 0.097,
            Sfx::Eject => 0.021,
            Sfx::Cartridge(_) => 0.0,
        }
    }

    pub fn tail(self) -> f32 {
        let pcm = match self {
            Sfx::PowerSwitch => POWER_SWITCH,
            Sfx::Insert => INSERT,
            Sfx::Eject => EJECT,
            Sfx::Cartridge(v) => CAROUSEL[v as usize % CAROUSEL.len()],
        };
        pcm.len() as f32 / 2.0 / ASSET_HZ - self.lead()
    }

    pub fn render(self, sample_rate: u32) -> Vec<i16> {
        let pcm = match self {
            Sfx::PowerSwitch => POWER_SWITCH,
            Sfx::Insert => INSERT,
            Sfx::Eject => EJECT,
            Sfx::Cartridge(v) => CAROUSEL[v as usize % CAROUSEL.len()],
        };
        let mut out = Vec::with_capacity(pcm.len());
        // Carousel impacts should suggest quiet plastic movement, not a sharp
        // tabletop strike. Two gentle low-pass poles and -6 dB gain soften only
        // these clips; the power switch and insert/eject samples stay unchanged.
        let soft = matches!(self, Sfx::Cartridge(_));
        let pole = 1.0 - (-std::f32::consts::TAU * 3200.0 / sample_rate.max(1) as f32).exp();
        let (mut first, mut second) = (0.0, 0.0);
        for mut v in resampled(pcm, sample_rate) {
            if soft {
                first += pole * (v - first);
                second += pole * (first - second);
                v = second * 0.5;
            }
            let s = v.clamp(-32768.0, 32767.0) as i16;
            out.push(s);
            out.push(s);
        }
        out
    }
}

fn resampled(pcm: &[u8], sample_rate: u32) -> Vec<f32> {
    let src: Vec<f32> = pcm
        .chunks_exact(2)
        .map(|b| i16::from_le_bytes([b[0], b[1]]) as f32)
        .collect();
    if sample_rate == ASSET_HZ as u32 || src.is_empty() {
        return src;
    }
    let ratio = sample_rate as f32 / ASSET_HZ;
    let n = (src.len() as f32 * ratio) as usize;
    (0..n)
        .map(|i| {
            let x = i as f32 / ratio;
            let a = x as usize;
            let f = x - a as f32;
            let b = (a + 1).min(src.len() - 1);
            src[a] * (1.0 - f) + src[b] * f
        })
        .collect()
}
