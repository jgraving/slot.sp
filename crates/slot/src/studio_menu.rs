//! Upstream Studio controls, with slot.sp's card, fonts, hints and cartridge renderer.
use crate::{
    art::{self, Artwork},
    studio::{Effect, LabelKind, Row, Screen, Studio, Target},
};
use slot_input::{Action, Btn};
use slot_store::Cart;
use slot_ui::{Canvas, CartFace, TexId, UndoFace};
use std::{
    path::{Path, PathBuf},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
};

pub struct Menu {
    pub model: Studio,
    pub face: Option<TexId>,
    pub art_face: Option<TexId>,
    pub visible: bool,
    root: PathBuf,
    original: Option<Arc<Vec<u8>>>,
    art: Arc<Artwork>,
    jobs: Sender<Job>,
    done: Receiver<Done>,
    fetch: Option<Receiver<Result<Artwork, crate::update::UpdateFail>>>,
    preview: Option<CartFace>,
    generation: u64,
    preview_generation: u64,
    saved: Option<Cart>,
    saving: bool,
    notice: String,
}
enum Job {
    Preview(u64, Studio, Arc<Artwork>, Option<Arc<Vec<u8>>>),
    Save(Studio, Arc<Artwork>, Option<Arc<Vec<u8>>>),
}
enum Done {
    Preview(u64, CartFace),
    Saved(Result<Cart, std::io::Error>),
}
impl Menu {
    pub fn new(root: PathBuf, cart: Cart, wifi: bool) -> Self {
        let original = cart
            .label
            .as_ref()
            .and_then(|p| std::fs::read(p).ok())
            .map(Arc::new);
        let mut model = Studio::new(cart, wifi);
        match original
            .as_deref()
            .and_then(|b| slot_ui::house_label::label_tag(b))
        {
            Some(slot_ui::house_label::LabelTag::Logo(c)) => model.adopt_logo(c),
            Some(slot_ui::house_label::LabelTag::Text(c)) => model.adopt_text(c),
            None => {}
        }
        let (jobs, rx) = channel();
        let (tx, done) = channel();
        let card = root.clone();
        std::thread::spawn(move || {
            while let Ok(first) = rx.recv() {
                let mut newest = None;
                let mut saving = None;
                for job in std::iter::once(first).chain(rx.try_iter()) {
                    match job {
                        Job::Preview(..) => newest = Some(job),
                        Job::Save(..) => saving = Some(job),
                    }
                }
                let Some(job) = saving.or(newest) else {
                    continue;
                };
                let result = match job {
                    Job::Preview(gen, model, art, original) => Done::Preview(
                        gen,
                        preview(&model, &art, original.as_deref().map(Vec::as_slice)),
                    ),
                    Job::Save(model, art, original) => Done::Saved(save(
                        &card,
                        &model,
                        &art,
                        original.as_deref().map(Vec::as_slice),
                    )),
                };
                if tx.send(result).is_err() {
                    break;
                }
            }
        });
        let mut menu = Self {
            model,
            face: None,
            art_face: None,
            visible: true,
            root,
            original,
            art: Arc::default(),
            jobs,
            done,
            fetch: None,
            preview: None,
            generation: 0,
            preview_generation: 0,
            saved: None,
            saving: false,
            notice: String::new(),
        };
        menu.rebuild();
        menu
    }
    pub fn preview_ready(&self) -> bool {
        self.preview_generation == self.generation
    }
    pub fn focus_info(&self)->Option<(usize,usize)> {
        match self.model.screen {Screen::List{row}=>Some((row,0)),Screen::Sheet{at}=>Some((at,0)),Screen::Custom{channel,..}=>Some((channel,0)),_=>None}
    }
    pub fn page_key(&self) -> u64 {
        4100 + match self.model.screen {
            Screen::List { .. } => 0,
            Screen::Sheet { .. } => 1,
            Screen::Chips { .. } => 2,
            Screen::Custom { .. } => 3,
            Screen::Hex { .. } => 4,
        }
    }
    pub fn notice_text(&self) -> Option<&str> {
        (!self.notice.is_empty()).then_some(&self.notice)
    }
    pub fn take_saved(&mut self) -> Option<Cart> {
        self.saved.take()
    }
    pub fn finish(&mut self) {
        if self.saving {
            return;
        }
        self.saving = true;
        self.notice = "Saving cartridge…".into();
        if self
            .jobs
            .send(Job::Save(
                self.model.clone(),
                self.art.clone(),
                self.original.clone(),
            ))
            .is_err()
        {
            self.saving = false;
            self.notice = "Editor unavailable".into();
        }
    }
    pub fn busy(&self) -> bool {
        self.saving
    }
    fn rebuild(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.face = None;
        let _ = self.jobs.send(Job::Preview(
            self.generation,
            self.model.clone(),
            self.art.clone(),
            self.original.clone(),
        ));
    }
    pub fn update(&mut self) {
        if let Some(result) = self.fetch.as_ref().and_then(|rx| rx.try_recv().ok()) {
            self.fetch = None;
            match result {
                Ok(art) => {
                    self.model
                        .art_arrived(art.real.is_some(), art.logo.is_some());
                    if art.wheel.is_some() {
                        self.model.wheel_arrived();
                    }
                    if art.real.is_none() && art.logo.is_none() {
                        self.notice = "No matching artwork".into();
                    } else {
                        self.notice.clear();
                    }
                    self.art = Arc::new(art);
                }
                Err(error) => {
                    self.model.art_failed = true;
                    self.model.art_arrived(false, false);
                    self.notice = if error == crate::update::UpdateFail::Clock {
                        "Set the clock, then retry"
                    } else {
                        "Artwork unavailable · reopen to retry"
                    }
                    .into();
                }
            }
            self.rebuild();
        }
        while let Ok(done) = self.done.try_recv() {
            match done {
                Done::Preview(gen, face) if gen == self.generation => {
                    self.preview = Some(face);
                    self.preview_generation = gen;
                    self.face = None;
                }
                Done::Saved(Ok(cart)) => {
                    self.saved = Some(cart);
                    self.saving = false;
                    self.visible = false;
                }
                Done::Saved(Err(_)) => {
                    self.saving = false;
                    self.notice = "Could not save · try again".into();
                    self.face = None;
                }
                _ => {}
            }
        }
    }
    pub fn input(&mut self, action: Action) {
        if self.saving {
            return;
        }
        let btn = match action {
            Action::GbaDown(btn) => btn,
            Action::QuickMenu => Btn::B,
            _ => return,
        };
        self.notice.clear();
        let before = self.model.clone();
        match self.model.press(btn) {
            Some(Effect::Close) => {
                self.finish();
            }
            Some(Effect::Fetch) if !self.model.art_ready() && self.fetch.is_none() => {
                let (tx, rx) = channel();
                let (root, cart) = (self.root.clone(), self.model.cart.clone());
                std::thread::spawn(move || {
                    let _ = tx.send(art::fetch(&root, &cart));
                });
                self.fetch = Some(rx);
                self.notice = "Finding labels…".into();
            }
            Some(Effect::Fetch) if self.art.real.is_none() && self.art.logo.is_none() => {
                self.notice = "More labels need Wi-Fi".into();
            }
            Some(Effect::Apply(kind)) => self.model.applied(kind),
            _ => {}
        }
        if before != self.model {
            self.rebuild();
        } else {
            self.face = None;
        }
    }
    /// Cartridge pixels and palette swatches bypass semantic light-theme recoloring.
    pub fn artwork_face(&self, canvas: Canvas) -> UndoFace {
        let mut face = UndoFace {
            rgba: vec![0; (canvas.w as u32 * 480 * 4) as usize],
            w: canvas.w as u32,
            h: 480,
        };
        let left = 76.0;
        let edge = canvas.w * 0.47;
        if let Some(cart) = &self.preview {
            let scale = ((edge - left - 12.0) / cart.w as f32).min(210.0 / cart.h as f32);
            let src = UndoFace {
                rgba: cart.rgba.clone(),
                w: cart.w,
                h: cart.h,
            };
            place(
                &mut face,
                &src,
                left + (edge - left - cart.w as f32 * scale) / 2.0,
                102.0 + (210.0 - cart.h as f32 * scale) / 2.0,
                scale,
            );
        }
        if let Screen::Chips { at, .. } = self.model.screen {
            use slot_ui::palette::{chip_grid, Chip};
            let grid = chip_grid(0.0, 0.0);
            let x = edge + 12.0;
            let w = canvas.w - 76.0 - x;
            let top = (grid[at].y - 108.0).max(0.0);
            let scale = (w / 264.0).min(0.86);
            for p in &grid {
                let yy = 119.0 + (p.y - top) * scale;
                if !(113.0..=298.0).contains(&yy) {
                    continue;
                }
                let xx = x + p.x * scale;
                let sz = 42.0 * scale;
                if let Chip::Preset(n) = p.chip {
                    let c = slot_ui::shell_presets()[n].1.colour;
                    slot_ui::expressive::raster_round(
                        &mut face,
                        xx,
                        yy,
                        sz,
                        sz,
                        sz / 2.0,
                        [
                            c[0] as f32 / 255.0,
                            c[1] as f32 / 255.0,
                            c[2] as f32 / 255.0,
                            1.0,
                        ],
                    );
                }
            }
        }
        face
    }
    pub fn screen_face(&self, canvas: Canvas) -> UndoFace {
        let mut face = UndoFace {
            rgba: vec![0; (canvas.w as u32 * 480 * 4) as usize],
            w: canvas.w as u32,
            h: 480,
        };
        let title = slot_ui::quick_heading_face("Cart Studio");
        place(&mut face, &title, canvas.centred(title.w as f32), 0.0, 1.0);
        let mode = match self.model.screen {
            Screen::List { .. } => 0,
            Screen::Sheet { .. } => 1,
            Screen::Chips { at, .. } => {
                if matches!(
                    slot_ui::palette::chip_grid(0.0, 0.0)[at].chip,
                    slot_ui::palette::Chip::Custom
                ) {
                    2
                } else {
                    5
                }
            }
            Screen::Custom { .. } => 3,
            Screen::Hex { .. } => 4,
        };
        let hints = slot_ui::cart_studio_legends(
            mode == 0,
            mode,
            matches!(self.model.row(), Some(Row::Finish | Row::Shape)),
        );
        if let Some(back) = hints.first() {
            place(&mut face, back, 16.0, 378.0, 1.0);
        }
        let total = hints.iter().skip(1).map(|h| h.w).sum::<u32>() as f32
            + 12.0 * hints.len().saturating_sub(2) as f32;
        let mut x = canvas.w - 16.0 - total;
        for hint in hints.iter().skip(1) {
            place(&mut face, hint, x, 378.0, 1.0);
            x += hint.w as f32 + 12.0;
        }
        let left = 76.0;
        let edge = canvas.w * 0.47;
        let right = canvas.w - 76.0;
        text(
            &mut face,
            &slot_ui::label_text(&self.model.cart),
            left,
            316.0,
            edge - left - 8.0,
            22.0,
            false,
        );
        let x = edge + 12.0;
        let w = right - x;
        match self.model.screen {
            Screen::List { row } => {
                let rows = self.model.rows();
                let pitch = (262.0 / rows.len().max(1) as f32).min(64.0);
                for (i, item) in rows.iter().enumerate() {
                    let name = match item {
                        Row::Label => "Label",
                        Row::LabelColor => "Label color",
                        Row::Color => "Shell color",
                        Row::Finish => "Finish",
                        Row::Shape => "Shape",
                    };
                    let value = match item {
                        Row::Finish => match self.model.current_shell().finish {
                            slot_store::ShellFinish::Solid => "Solid",
                            slot_store::ShellFinish::Clear => "Clear",
                            slot_store::ShellFinish::Glitter => "Glitter",
                        }
                        .into(),
                        Row::Shape => match self.model.current_shell().outline {
                            slot_store::Outline::Auto => "Automatic",
                            slot_store::Outline::Notched => "Notched",
                            slot_store::Outline::Rounded => "Rounded",
                        }
                        .into(),
                        Row::Color => rgb(self.model.current_shell().colour),
                        Row::LabelColor => self
                            .model
                            .ground
                            .map(rgb)
                            .unwrap_or_else(|| "Automatic".into()),
                        _ => "Choose…".into(),
                    };
                    let y = 82.0 + i as f32 * pitch;
                    text(&mut face, name, x, y, w, 24.0, i == row);
                    text(&mut face, &value, x, y + 27.0, w, 20.0, false);
                }
            }
            Screen::Sheet { at } => {
                for (i, kind) in self.model.sheet.iter().enumerate() {
                    let name = match kind {
                        LabelKind::Existing => "Original label",
                        LabelKind::Real => "Scanned label",
                        LabelKind::Logo => "Logo only",
                        LabelKind::Title => "Title only",
                    };
                    let y = 96.0 + i as f32 * 60.0;
                    text(&mut face, name, x, y, w, 24.0, i == at);
                }
            }
            Screen::Chips { target, at } => {
                use slot_ui::palette::{chip_grid, Chip};
                let grid = chip_grid(0.0, 0.0);
                text(
                    &mut face,
                    if target == Target::Shell {
                        "Shell color"
                    } else {
                        "Label color"
                    },
                    x,
                    78.0,
                    w,
                    24.0,
                    true,
                );
                let selected_y = grid[at].y;
                let top = (selected_y - 108.0).max(0.0);
                let scale = (w / 264.0).min(0.86);
                for (i, p) in grid.iter().enumerate() {
                    let yy = 119.0 + (p.y - top) * scale;
                    if !(113.0..=298.0).contains(&yy) {
                        continue;
                    }
                    let xx = x + p.x * scale;
                    let sz = 42.0 * scale;
                    highlight(&mut face, xx - 3.0, yy - 3.0, sz + 6.0, sz + 6.0, i == at);
                    if !matches!(p.chip, Chip::Preset(_)) {
                        text(
                            &mut face,
                            match p.chip {
                                Chip::Current => "O",
                                Chip::Auto => "A",
                                _ => "+",
                            },
                            xx,
                            yy + 2.0,
                            sz,
                            22.0,
                            i == at,
                        );
                    }
                }
                let name = match grid[at].chip {
                    Chip::Preset(n) => slot_ui::shell_presets()[n].0,
                    Chip::Current => "Original",
                    Chip::Auto => "Automatic",
                    Chip::Custom => "Custom…",
                };
                text(&mut face, name, x, 324.0, w, 22.0, true);
            }
            Screen::Custom {
                target: _,
                rgb,
                channel,
                ..
            } => {
                text(&mut face, "Custom color", x, 80.0, w, 24.0, true);
                for i in 0..3 {
                    let y = 132.0 + i as f32 * 52.0;
                    text(
                        &mut face,
                        &format!("{}  ‹ {} ›", ["R", "G", "B"][i], rgb[i]),
                        x,
                        y,
                        w,
                        26.0,
                        i == channel,
                    );
                }
                text(&mut face, &self::rgb(rgb), x, 310.0, w, 22.0, false);
            }
            Screen::Hex { digits, pos, .. } => {
                text(&mut face, "Hex color", x, 98.0, w, 26.0, true);
                let step = w / 6.0;
                for (i, d) in digits.iter().enumerate() {
                    let xx = x + i as f32 * step;
                    highlight(&mut face, xx, 166.0, step - 3.0, 52.0, i == pos);
                    text(
                        &mut face,
                        &format!("{d:X}"),
                        xx,
                        174.0,
                        step - 3.0,
                        28.0,
                        i == pos,
                    );
                }
            }
        }
        face
    }
}
fn rgb(c: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2])
}
fn highlight(face: &mut UndoFace, x: f32, y: f32, w: f32, h: f32, on: bool) {
    if on {
        slot_ui::expressive::raster_round(face, x, y, w, h, 12.0, slot_ui::expressive::SELECTED);
    }
}
fn text(dst: &mut UndoFace, value: &str, x: f32, y: f32, w: f32, px: f32, lit: bool) {
    if let Some(font) = slot_ui::text::font(if lit {
        slot_ui::text::Role::Focus
    } else {
        slot_ui::text::Role::Menu
    }) {
        let layout = slot_ui::text::fit_preserving_case(font, value, w, 2, px, px - 2.0);
        let mut src = UndoFace {
            rgba: vec![0; (w as u32 * 52 * 4) as usize],
            w: w as u32,
            h: 52,
        };
        slot_ui::text::draw_centred(
            &mut src.rgba,
            src.w,
            src.h,
            &layout,
            if lit {
                [244, 241, 234]
            } else {
                [180, 196, 194]
            },
        );
        place(dst, &src, x, y - 9.0, 1.0);
    }
}
fn place(dst: &mut UndoFace, src: &UndoFace, x: f32, y: f32, scale: f32) {
    let (w, h) = ((src.w as f32 * scale) as u32, (src.h as f32 * scale) as u32);
    for yy in 0..h {
        for xx in 0..w {
            let (dx, dy) = (x as u32 + xx, y as u32 + yy);
            if dx >= dst.w || dy >= dst.h {
                continue;
            }
            let s = ((((yy as f32 / scale) as u32).min(src.h - 1) * src.w
                + ((xx as f32 / scale) as u32).min(src.w - 1))
                * 4) as usize;
            let d = ((dy * dst.w + dx) * 4) as usize;
            let a = src.rgba[s + 3] as u32;
            let b = dst.rgba[d + 3] as u32 * (255 - a) / 255;
            let total = a + b;
            if total == 0 {
                continue;
            }
            for c in 0..3 {
                dst.rgba[d + c] =
                    ((src.rgba[s + c] as u32 * a + dst.rgba[d + c] as u32 * b) / total) as u8;
            }
            dst.rgba[d + 3] = total as u8;
        }
    }
}
fn label(model: &Studio, art: &Artwork, original: Option<&[u8]>) -> Option<Vec<u8>> {
    match model.shown_label() {
        LabelKind::Existing => {
            let mut edited = model.cart.clone();
            edited.shell = model.shell;
            if model.wearing_text()
                && slot_ui::label_size(&edited) != slot_ui::label_size(&model.cart)
                && original.is_some()
            {
                // A saved title is generated artwork: lay its text out again for
                // the new Game Boy shape instead of cropping the old rectangle.
                model.preview_ground().map(|c| art::make::text_label(&edited, c))
            } else {
                original.map(<[u8]>::to_vec)
            }
        }
        LabelKind::Real => art.real.clone(),
        LabelKind::Logo => {
            let look = model
                .ground
                .map(|ground| art::index::Look {
                    ground,
                    band: art.look.as_ref().and_then(|l| l.band),
                })
                .or_else(|| art.look.clone());
            art.wheel
                .as_deref()
                .and_then(|b| {
                    art::make::logo_label(b, look.as_ref(), &model.cart.stem, model.cart.platform)
                })
                .or_else(|| art.logo.clone())
        }
        LabelKind::Title => model.preview_ground().map(|c| {
            let mut edited=model.cart.clone();edited.shell=model.shell;
            art::make::text_label(&edited,c)
        }),
    }
}
fn preview(model: &Studio, art: &Artwork, original: Option<&[u8]>) -> CartFace {
    let mut cart = model.cart.clone();
    cart.shell = model.shell;
    let bytes = label(model, art, original);
    let (w,h)=slot_ui::label_size(&cart);
    let art=bytes.as_deref().and_then(|b|slot_ui::cover_label_bytes(b,w,h));
    let mut face = slot_ui::cart_face_with(&cart, art);
    if slot_ui::shows_interior(&cart) {
        let mut under=slot_ui::cart_interior(&cart);
        composite(&mut under,&slot_ui::cart_backing(slot_ui::gb_shell_of(&cart)));
        composite(&mut under,&face);
        face=under;
    }
    face
}
fn composite(bottom:&mut CartFace,top:&CartFace) {
    for (bg,px) in bottom.rgba.chunks_exact_mut(4).zip(top.rgba.chunks_exact(4)) {
        let a=px[3]as u32;let b=bg[3]as u32*(255-a)/255;let total=a+b;if total==0 {continue;}
        for c in 0..3 {bg[c]=((px[c]as u32*a+bg[c]as u32*b)/total)as u8;}bg[3]=total as u8;
    }
}
pub fn save(
    root: &Path,
    model: &Studio,
    art: &Artwork,
    original: Option<&[u8]>,
) -> std::io::Result<Cart> {
    // scan() resolves label filenames case-insensitively. Replace/remove that
    // exact file instead of creating a competing canonical name on Linux cards.
    let path = model
        .cart
        .label
        .clone()
        .unwrap_or_else(|| art::make::label_path(root, &model.cart));
    // Opening the editor may have hit a temporary read error. Retry on this
    // worker before writing anything; an absent snapshot must not erase a label
    // or make rollback impossible if the shell write fails.
    let recovered = if original.is_none() && model.cart.label.is_some() {
        Some(std::fs::read(&path)?)
    } else {
        None
    };
    let original = original.or(recovered.as_deref());
    let png = label(model, art, original);
    if let Some(png) = &png {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        slot_store::atomic_write(&path, png)?;
    } else if path.exists() {
        std::fs::remove_file(&path)?;
    }
    if let Err(error) =
        slot_store::cart_shell::write_choice(root, &model.cart.stem, model.shell.as_ref())
    {
        if let Some(original) = original {
            let _ = slot_store::atomic_write(&path, original);
        } else {
            let _ = std::fs::remove_file(&path);
        }
        return Err(error);
    }
    let _ = std::fs::remove_file(crate::label_cache::cache_path(root, &model.cart));
    let mut cart = model.cart.clone();
    cart.shell = model.shell;
    cart.label = png.map(|_| path);
    Ok(cart)
}
