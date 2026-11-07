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

use crate::studio_gallery::{self, Gallery};

pub struct Menu {
    pub gallery_tex: Option<TexId>,
    gallery: Option<Gallery>,
    now: u64,
    printed_at: u64,
    applying: Option<(LabelKind, u64, usize)>,
    landed: Option<LabelKind>,
    pub model: Studio,
    pub face: Option<TexId>,
    pub art_face: Option<TexId>,
    pub cart_face: Option<TexId>,
    cart_dirty: bool,
    opened_at: Option<u64>,
    pub source_rect: Option<[f32;4]>,
    clean: Studio,
    close_choice: Option<usize>,
    close_after_save: bool,
    pub visible: bool,
    root: PathBuf,
    original: Option<Arc<Vec<u8>>>,
    art: Arc<Artwork>,
    jobs: Sender<Job>,
    done: Receiver<Done>,
    fetch: Option<Receiver<Result<(Artwork, bool), crate::update::UpdateFail>>>,
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
    Preview(u64, CartFace, Gallery),
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
                        gallery(&model, &art, original.as_deref().map(Vec::as_slice)),
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
            clean: model.clone(), close_choice: None, close_after_save: false, source_rect: None,
            model,
            gallery_tex: None, gallery: None, now: 0, printed_at: 0, applying: None, landed: None,
            face: None,
            art_face: None,
            cart_face: None, cart_dirty: false, opened_at: None,
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
        if let Some(at)=self.close_choice {return Some((at,0));}
        match self.model.screen {Screen::List{row}=>Some((row,0)),Screen::Sheet{at}=>Some((at,0)),Screen::Custom{channel,..}=>Some((channel,0)),_=>None}
    }
    pub fn page_key(&self) -> u64 {
        if self.close_choice.is_some() {return 4105;}
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
    pub fn has_unsaved_changes(&self) -> bool {
        self.applying.is_some() || self.model.current_shell()!=self.clean.current_shell()
            || self.model.wearing!=self.clean.wearing
            || self.model.preview_ground()!=self.clean.preview_ground()
    }
    pub fn confirming_close(&self) -> bool {self.close_choice.is_some()}
    pub fn option_pitch(&self) -> f32 {(240.0/self.model.rows().len().saturating_sub(1).max(1) as f32).min(60.0)}
    pub fn finish(&mut self) {self.save_changes(true);}
    fn save_changes(&mut self, close: bool) {
        if self.saving {
            return;
        }
        self.complete_apply();
        self.close_choice=None;
        self.close_after_save=close;
        if !self.has_unsaved_changes() {
            self.visible=!close;self.notice="Saved".into();self.face=None;return;
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
    pub(crate) fn refresh_shell(&mut self) { self.rebuild(); }
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
    pub fn update(&mut self) { self.update_at(self.now); }
    pub fn update_at(&mut self, now: u64) {
        self.now = now;
        self.opened_at.get_or_insert(now);
        if self.applying.is_some_and(|(_, at, _)| now.saturating_sub(at) >= studio_gallery::APPLY_MS) {
            self.complete_apply();
        }
        if let Some(result) = self.fetch.as_ref().and_then(|rx| rx.try_recv().ok()) {
            self.fetch = None;
            match result {
                Ok((art, duplicates)) => {
                    self.model.duplicates = duplicates;
                    self.model
                        .art_arrived(art.real.is_some(), art.logo.is_some());
                    if art.wheel.is_some() {
                        self.model.wheel_arrived();
                    }
                    if art.real.is_none() && art.logo.is_none() {
                        self.notice = if duplicates { "No new labels" } else { "No matching artwork" }.into();
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
                Done::Preview(gen, face, gallery) if gen == self.generation => {
                    self.gallery = Some(gallery);
                    self.landed = None;
                    self.printed_at = self.now;
                    self.preview = Some(face);
                    self.cart_dirty = true;
                    self.preview_generation = gen;
                    self.face = None;
                }
                Done::Saved(Ok(cart)) => {
                    self.saved = Some(cart);
                    self.clean=self.model.clone();
                    self.saving = false;
                    self.visible = !self.close_after_save;
                    self.notice="Saved".into();self.face=None;
                    // A save can supersede a coalesced preview job. Refresh it
                    // when staying open so the latest label cannot stay stale.
                    if self.visible {self.rebuild();}
                }
                Done::Saved(Err(_)) => {
                    self.saving = false;
                    self.notice = "Could not save · try again".into();
                    self.rebuild();
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
        if let Some(at)=self.close_choice {
            match btn {
                Btn::Up=>self.close_choice=Some((at+2)%3),
                Btn::Down=>self.close_choice=Some((at+1)%3),
                Btn::B=>self.close_choice=None,
                Btn::A=>match at {
                    0=>self.save_changes(true),
                    1=>{self.visible=false;self.close_choice=None;},
                    _=>self.close_choice=None,
                }, _=>{},
            }
            self.face=None;return;
        }
        if self.applying.is_some() { return; }
        self.notice.clear();
        let before = self.model.clone();
        match self.model.press(btn) {
            Some(Effect::Close) => {
                if self.has_unsaved_changes() {self.close_choice=Some(0);}
                else {self.visible=false;}
            }
            Some(Effect::Save) => self.save_changes(false),
            Some(Effect::Fetch) if (!self.model.art_ready() || self.model.art_failed) && self.fetch.is_none() => {
                self.model.retry_art();
                let (tx, rx) = channel();
                let (root, cart, original) = (self.root.clone(), self.model.cart.clone(), self.original.clone());
                std::thread::spawn(move || {
                    let result = art::fetch(&root, &cart).map(|mut art| {
                        let duplicates = art.without_duplicates(original.as_deref().map(Vec::as_slice));
                        (art, duplicates)
                    });
                    let _ = tx.send(result);
                });
                self.fetch = Some(rx);
                self.notice = "Finding labels…".into();
            }
            Some(Effect::Fetch) if self.art.real.is_none() && self.art.logo.is_none() => {
                self.notice = if !self.model.wifi_available() { "More labels need Wi-Fi" }
                    else if self.model.duplicates { "No new labels" }
                    else { "No matching artwork" }.into();
            }
            Some(Effect::Apply(kind)) => {
                if let Screen::Sheet { at } = self.model.screen {
                    if self.gallery.as_ref().is_some_and(|g| g.slots.iter().any(|(k,_,_)| *k == kind)) {
                        self.applying = Some((kind, self.now, at));
                    }
                }
            },
            _ => {}
        }
        if before.shell != self.model.shell || before.ground != self.model.ground
            || before.wearing != self.model.wearing { self.rebuild(); }
        else { self.face = None; }
    }
    pub fn entry(&self) -> f32 {
        studio_gallery::spring(self.now.saturating_sub(self.opened_at.unwrap_or(self.now)), 340.0, 0.86, 720)
    }
    pub fn take_cart_pixels(&mut self) -> Option<UndoFace> {
        if !self.cart_dirty { return None; }
        self.cart_dirty = false;
        self.preview.as_ref().map(|f| UndoFace { rgba:f.rgba.clone(), w:f.w, h:f.h })
    }
    pub fn cart_rect(&self, canvas: Canvas) -> [f32;4] {
        let (w,h) = self.preview.as_ref().map(|f|(f.w,f.h))
            .unwrap_or_else(||slot_ui::cart_box(self.model.cart.platform));
        let edge=canvas.w*0.47;
        let scale=((edge-76.0-12.0)/w as f32).min(210.0/h as f32);
        [76.0+(edge-76.0-w as f32*scale)/2.0, 102.0+(210.0-h as f32*scale)/2.0,
            w as f32*scale, h as f32*scale]
    }
    pub fn shown_cart_rect(&self, canvas: Canvas) -> [f32;4] {
        let to=self.cart_rect(canvas);
        studio_gallery::entry_rect(self.source_rect.unwrap_or(to),to,self.entry())
    }
    pub fn draw_title(&self, canvas: Canvas, tex: TexId, out: &mut Vec<slot_ui::Draw>) {
        let x=76.0; let w=canvas.w*0.47-x;
        out.push(slot_ui::Draw::Part { x,y:308.0,w,h:52.0,tex,alpha:self.entry().clamp(0.0,1.0),
            uv:[x/canvas.w,308.0/canvas.h,w/canvas.w,52.0/canvas.h] });
    }
    pub fn take_gallery_pixels(&mut self) -> Option<UndoFace> {
        self.gallery.as_mut()?.pixels.take()
    }
    fn complete_apply(&mut self) {
        if let Some((kind, _, _)) = self.applying.take() {
            self.model.applied(kind);
            self.landed = Some(kind);
            self.rebuild();
        }
    }
    pub fn applying(&self) -> bool { self.applying.is_some() }
    pub fn draw_gallery(&self, canvas: Canvas, frame: slot_ui::MenuFrame, out: &mut Vec<slot_ui::Draw>) {
        let (Some(gallery), Some(tex)) = (&self.gallery, self.gallery_tex) else { return };
        let Screen::Sheet { .. } = self.model.screen else { return };
        let begin = out.len();
        for (index, kind) in self.model.sheet.iter().copied().enumerate() {
            let Some((_, w, h)) = gallery.slots.iter().find(|(k,_,_)| *k == kind) else { continue };
            let mut rect = Gallery::rect(canvas, index, *w, *h);
            let elapsed = self.now.saturating_sub(self.printed_at).saturating_sub(index as u64 * 100);
            let feed = studio_gallery::spring(elapsed, 420.0, 0.74, 620);
            rect[1] -= (1.0 - feed) * 24.0;
            let opacity=(elapsed as f32/140.0).clamp(0.0,1.0);
            if !self.applying.is_some_and(|(selected,_,_)|selected==kind) {
                gallery.draw(tex, kind, rect, opacity, out);
            }
        }
        if let Some((kind, started, index)) = self.applying.or_else(|| self.landed.map(|k| (k, 0, 0))) {
            if let (Some((_, w, h)), Some(_cart)) = (gallery.slots.iter().find(|(k,_,_)| *k == kind), &self.preview) {
                let from = Gallery::rect(canvas, index, *w, *h);
                let [cx,cy,cw,ch] = self.shown_cart_rect(canvas);
                let mut edited = self.model.cart.clone(); edited.shell = self.model.shell;
                let (x,y,w,h) = slot_ui::label_rect(slot_ui::gb_shell_of(&edited));
                let to = [cx + x * cw, cy + y * ch,
                    w * cw, h * ch];
                let elapsed = if self.applying.is_some() { self.now.saturating_sub(started) } else { studio_gallery::APPLY_MS };
                let rect = studio_gallery::application_rect(from, to, elapsed);
                gallery.draw(tex, kind, rect, 1.0, out);
            }
        }
        studio_gallery::transform(canvas, frame, out, begin);
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
        let mode = if self.close_choice.is_some() {7} else if self.model.row()==Some(Row::Save) {6} else { match self.model.screen {
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
        }};
        let hints = slot_ui::cart_studio_legends(
            matches!(self.model.screen,Screen::List{..}) && self.close_choice.is_none(),
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
        if let Some(at)=self.close_choice {
            text(&mut face,"Save changes?",x,84.0,w,24.0,true);
            for (i,name) in ["Save and close","Discard changes","Keep editing"].iter().enumerate() {
                text(&mut face,name,x,144.0+i as f32*60.0,w,22.0,i==at);
            }
            return face;
        }
        match self.model.screen {
            Screen::List { row } => {
                let rows = self.model.rows();
                let pitch = self.option_pitch();
                for (i, item) in rows.iter().enumerate() {
                    let name = match item {
                        Row::Label => "Label",
                        Row::LabelColor => "Label color",
                        Row::Color => "Shell color",
                        Row::Finish => "Finish",
                        Row::Shape => "Shape",
                        Row::Save => "Save",
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
                    let y = if *item==Row::Save {330.0} else {76.0+i as f32*pitch};
                    if *item==Row::Save {
                        text(&mut face,"Save",x,y,w,24.0,i==row);continue;
                    }
                    let adjustable=matches!(item,Row::Finish|Row::Shape);
                    let inset=if adjustable {20.0}else{0.0};
                    text(&mut face, name, x+inset, y, w-20.0-inset, 22.0, i == row);
                    text(&mut face, &value, x+inset, y + 24.0, w-20.0-inset, 18.0, false);
                    let right=slot_ui::quick_caret_face(true);
                    place(&mut face,&right,x+w-right.w as f32,y+10.0,1.0);
                    if adjustable {
                        let left=slot_ui::quick_caret_face(false);
                        place(&mut face,&left,x,y+10.0,1.0);
                    }
                }
            }
            Screen::Sheet { at } => {
                text(&mut face, "Label printer", x, 76.0, w, 24.0, true);
                // A small paper slot sits above the two label previews.
                slot_ui::expressive::raster_round(&mut face, x + 8.0, 108.0, w - 16.0, 4.0, 2.0, slot_ui::expressive::SELECTED);
                for (i, kind) in self.model.sheet.iter().enumerate() {
                    let name = match kind {
                        LabelKind::Existing => "Original label",
                        LabelKind::Real => "Scanned label",
                        LabelKind::Logo => "Logo only",
                        LabelKind::Title => "Title only",
                    };
                    let y = 184.0 + i as f32 * studio_gallery::SHEET_PITCH;
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

fn gallery(model: &Studio, art: &Artwork, original: Option<&[u8]>) -> Gallery {
    let mut faces = Vec::new();
    for kind in [LabelKind::Existing, LabelKind::Real, LabelKind::Logo, LabelKind::Title] {
        let mut choice = model.clone();
        if kind != choice.wearing { choice.applied(kind); }
        let mut cart = choice.cart.clone(); cart.shell = choice.shell;
        let bytes = label(&choice, art, original).or_else(|| {
            (kind == LabelKind::Title || (kind == LabelKind::Existing && cart.label.is_none()))
                .then(|| art::make::text_label(&cart, slot_ui::label_colour(&slot_ui::label_text(&cart))))
        });
        let Some(bytes) = bytes else { continue };
        let (w,h) = slot_ui::label_size(&cart);
        let scale = (studio_gallery::CELL_W as f32 / w as f32)
            .min(studio_gallery::CELL_H as f32 / h as f32);
        let (w,h) = ((w as f32 * scale).floor().max(1.0) as u32,
            (h as f32 * scale).floor().max(1.0) as u32);
        if let Some(rgba) = slot_ui::cover_label_bytes(&bytes,w,h) {
            faces.push((kind, UndoFace { rgba,w,h }));
        }
    }
    Gallery::new(faces)
}
