//! Small, interruptible spatial springs and tonal surfaces. Everything here
//! changes existing quads; text is still rasterised only when its content changes.
use crate::{quick_row_top_at, Draw, QuickRow};

pub const ACCENT: [f32; 4] = [0.33, 0.76, 0.72, 1.0];
pub const MENU_INSET: f32 = 56.0;
pub const ROW_RADIUS: f32 = 12.0;
pub const ROW_H: f32 = 48.0;
pub const SELECTED: [f32; 4] = [0.20, 0.31, 0.32, 1.0];
pub const CARD_TOP: f32 = 56.0;
pub const CARD_BOTTOM: f32 = 366.0;
pub const ROW_TOP: f32 = 80.0;
pub const TITLE_H: f32 = 40.0;
pub const FOOTER_Y: f32 = 378.0;
pub const SURFACE: [f32;4] = [0.09, 0.115, 0.125, 1.0];
pub fn card_rect(canvas:crate::Canvas)->[f32;4] {[MENU_INSET,CARD_TOP,canvas.w-2.0*MENU_INSET,CARD_BOTTOM-CARD_TOP]}
pub fn draw_card(canvas:crate::Canvas,out:&mut Vec<Draw>) {let [x,y,w,h]=card_rect(canvas);rounded(out,x,y,w,h,28.0,SURFACE);}
pub fn is_chrome(d:&Draw)->bool {matches!(*d,Draw::Tex{y,h,..}|Draw::Part{y,h,..} if (y==0.0 && h==TITLE_H) || (y==FOOTER_Y && h<=44.0))}

/// Visible for the whole scroll session, including when the thumb is at either end.
pub fn draw_scrollbar(canvas:crate::Canvas,top:f32,visible:usize,total:usize,start:f32,out:&mut Vec<Draw>) {
    if total<=visible || visible==0 {return;}
    let h=crate::QUICK_PITCH*visible as f32-6.0;
    let thumb=(h*visible as f32/total as f32).max(24.0);
    let y=top+(h-thumb)*(start/(total-visible) as f32).clamp(0.0,1.0);
    let x=canvas.w-MENU_INSET-10.0;
    rounded(out,x,top,3.0,h,1.5,[0.55,0.64,0.65,0.22]);
    rounded(out,x,y,3.0,thumb,1.5,[0.76,0.84,0.83,1.0]);
}

pub fn rounded(out: &mut Vec<Draw>, x: f32, y: f32, w: f32, h: f32, radius: f32, colour: [f32; 4]) {
    if w > 0.0 && h > 0.0 && colour[3] > 0.0 {
        out.push(Draw::RoundRect {
            x,
            y,
            w,
            h,
            radius,
            colour,
        });
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Spring {
    pub(crate) value: f32,
    pub(crate) velocity: f32,
}
impl Spring {
    pub(crate) fn at(value: f32) -> Self {
        Self {
            value,
            velocity: 0.0,
        }
    }
    pub(crate) fn step(&mut self, target: f32, dt: f32, stiffness: f32, damping: f32) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }
        let dt = dt.min(1.0);
        let x = self.value - target;
        if x == 0.0 && self.velocity == 0.0 {
            return;
        }
        let omega = stiffness.sqrt();
        let (x, velocity) = if damping >= 1.0 {
            let b = self.velocity + omega * x;
            let decay = (-omega * dt).exp();
            (
                (x + b * dt) * decay,
                (self.velocity - omega * b * dt) * decay,
            )
        } else {
            let decay_rate = damping * omega;
            let frequency = omega * (1.0 - damping * damping).sqrt();
            let (sin, cos) = (frequency * dt).sin_cos();
            let decay = (-decay_rate * dt).exp();
            (
                (x * cos + (self.velocity + decay_rate * x) / frequency * sin) * decay,
                (self.velocity * cos
                    - (decay_rate * self.velocity + stiffness * x) / frequency * sin)
                    * decay,
            )
        };
        if x.abs() < 0.01 && velocity.abs() < 0.05 {
            *self = Self::at(target);
        } else {
            self.value = target + x;
            self.velocity = velocity;
        }
    }
}

/// The same critically damped spatial spring drives compact-to-expanded cards.
#[derive(Default, Debug)]
pub struct SurfaceMotion { compact: Spring, target:bool }
impl SurfaceMotion {
    pub fn snap(&mut self, compact: bool) { self.target=compact;self.compact=Spring::at(if compact {1.0}else{0.0}); }
    pub fn update(&mut self, compact: bool, dt: f32) {self.target=compact;self.advance(dt);}
    pub fn advance(&mut self,dt:f32) {self.compact.step(if self.target {1.0}else{0.0},dt,800.0,1.0);}
    pub fn value(&self) -> f32 { self.compact.value.clamp(0.0,1.0) }
}

#[derive(Clone, Copy, Debug)]
pub struct MenuFrame {
    pub focus_y: f32,
    pub grid_focus: [f32;2],
    pub tile_origin: Option<[f32;2]>,
    pub tile_reveal: f32,
    pub scroll_position: f32,
    pub window_start: f32,
    pub page_shift: f32,
    pub background: bool,
    pub outgoing: bool,
    pub entry: f32,
    pub opacity: f32,
    pub switch: Option<f32>,
    pub sliders: [Option<f32>; 2],
    pub focus_compact: f32,
    pub suppress_focus: bool,
    pub content_opacity: f32,
    pub closing: bool,
}
impl MenuFrame {
    pub fn settled(row: QuickRow, hidden: &[QuickRow]) -> Self {
        Self {
            focus_y: quick_row_top_at(row, row, hidden),
            grid_focus:crate::menu_grid::position(row,hidden),
            tile_origin:None,
            tile_reveal:1.0,
            scroll_position: row.position(hidden) as f32,
            window_start: crate::quick_window(row, hidden).0 as f32,
            page_shift: 0.0,
            background: true,
            outgoing: false,
            entry: 1.0,
            opacity: 1.0,
            switch: None,
            sliders: [None; 2],
            focus_compact: if row.page() == QuickRow::PLAY {
                1.0
            } else {
                0.0
            },
            suppress_focus: false,
            content_opacity: 1.0,
            closing: false,
        }
    }
}

#[derive(Default, Debug)]
pub struct MenuMotion {
    active: bool,
    grid_x: Spring,
    grid_y: Spring,
    tile_origin: Option<[f32;2]>,
    page: Option<u64>,
    custom: Option<u64>,
    outgoing_custom: bool,
    pause_morph: bool,
    pause_focus: Spring,
    compact: Spring,
    outgoing_page: Option<u64>,
    outgoing_focus_y: f32,
    target_scroll:f32,
    target_window:f32,
    target_focus:f32,
    row: Option<QuickRow>,
    scroll: Spring,
    window: Spring,
    slide: Spring,
    direction: f32,
    outgoing: Option<QuickRow>,
    entry: Spring,
    switch: Spring,
}
impl MenuMotion {
    pub fn update(
        &mut self,
        row: Option<QuickRow>,
        hidden: &[QuickRow],
        on: Option<bool>,
        dt: f32,
    ) {
        self.update_page(row, hidden, on, None, dt);
    }
    pub fn update_page(
        &mut self,
        row: Option<QuickRow>,
        hidden: &[QuickRow],
        on: Option<bool>,
        custom: Option<u64>,
        dt: f32,
    ) {
        self.update_page_at(row, hidden, on, custom, None, dt);
    }
    pub fn update_page_at(
        &mut self,
        row: Option<QuickRow>,
        hidden: &[QuickRow],
        on: Option<bool>,
        custom: Option<u64>,
        selection: Option<(usize, usize)>,
        dt: f32,
    ) {
        let Some(row) = row else {
            self.active = false;
            self.entry.step(0.0, dt, 800.0, 1.0);
            self.slide.step(1.0,dt,600.0,0.82);
            self.scroll.step(self.target_scroll,dt,800.0,0.76);
            self.window.step(self.target_window,dt,800.0,0.76);
            self.pause_focus.step(self.target_focus,dt,800.0,0.76);
            self.compact.step(if self.row.is_some_and(|r|r.page()==QuickRow::PLAY) {1.0}else{0.0},dt,800.0,1.0);
            if self.entry.value == 0.0 && self.entry.velocity == 0.0 {
                self.outgoing = None;
                self.page = None;
                self.row = None;
            }
            return;
        };
        self.active = true;
        let target = selection.map_or_else(|| row.position(hidden), |s| s.0) as f32;
        let window = selection.map_or_else(|| crate::quick_window(row, hidden).0, |s| s.1) as f32;
        self.target_scroll=target;self.target_window=window;self.target_focus=quick_row_top_at(row,row,hidden);
        let page = custom.unwrap_or(row.page()[0].index() as u64);
        if self.page != Some(page) {
            let opening = self.page.is_none();
            let old_root=self.row.is_some_and(|r|r.page()==QuickRow::MAIN)&&self.custom.is_none();
            let new_root=row.page()==QuickRow::MAIN&&custom.is_none();
            if old_root!=new_root && !opening {
                let root=if old_root {self.row.unwrap()}else{row};
                let i=root.position(hidden);
                self.tile_origin=Some([(i%4) as f32,(i/4) as f32]);
            }else{self.tile_origin=None;}
            if new_root {
                self.grid_x=Spring::at(crate::menu_grid::position(row,hidden)[0]);
                self.grid_y=Spring::at(crate::menu_grid::position(row,hidden)[1]);
            }
            let previous_frame = self.row.map(|old| self.frame_page(old,hidden,self.custom));
            let reversing=self.outgoing.is_some() && self.outgoing_page==Some(page);
            let was_morph=self.pause_morph && self.outgoing.is_some();
            let previous_direction=self.direction;
            self.pause_morph = self.custom.is_none()
                && custom.is_none()
                && self.row.is_some_and(|old| {
                    (old.page() == QuickRow::PLAY && row.page() == QuickRow::SETTINGS)
                        || (old.page() == QuickRow::SETTINGS && row.page() == QuickRow::PLAY)
                });
            if !was_morph {self.pause_focus=Spring::at(previous_frame.map_or(88.0,|f|f.focus_y));}
            self.outgoing_focus_y=previous_frame.map_or(88.0,|f|f.focus_y);
            self.outgoing_page=self.page;
            self.outgoing = self.row.filter(|_| !opening);
            self.outgoing_custom = self.custom.is_some();
            self.direction = if (self.custom.is_some() && custom.is_none())
                || (self.custom.zip(custom).is_some_and(|(old, new)| {let order=|k|if k==2010 {2001}else{k};order(new)<order(old)}))
                || self.row.is_some_and(|old| {
                    old.parent() == Some(row) || old.parent_in_game() == Some(row)
                }) {
                -1.0
            } else {
                1.0
            };
            if reversing {self.direction=-previous_direction;}
            self.page = Some(page);
            self.custom = custom;
            self.scroll = Spring::at(target);
            self.window = Spring::at(window);
            self.slide = if opening {
                Spring::at(1.0)
            } else if reversing {
                Spring{value:1.0-self.slide.value,velocity:-self.slide.velocity}
            } else {
                Spring::default()
            };
            if opening {
                self.compact=Spring::at(if row.page()==QuickRow::PLAY {1.0}else{0.0});
                self.entry = Spring::default();
            }
        }
        if self.row != Some(row) {
            self.switch = Spring::at(if on == Some(true) { 1.0 } else { 0.0 });
        }
        self.row = Some(row);
        if crate::menu_grid::is_page(row.page()) && custom.is_none() {
            self.grid_x.step(crate::menu_grid::position(row,hidden)[0],dt,800.0,0.76);
            self.grid_y.step(crate::menu_grid::position(row,hidden)[1],dt,800.0,0.76);
        }
        // Expressive fast spatial for focus and switches, default spatial for
        // page arrival. Critically damp opacity: it must never bounce.
        self.scroll.step(target, dt, 800.0, 0.76);
        self.window.step(window, dt, 800.0, 0.76);
        self.slide.step(1.0, dt, 600.0, 0.82);
        if self.slide.value == 1.0 && self.slide.velocity == 0.0 {
            self.outgoing = None;
        }
        self.entry.step(1.0, dt, 800.0, 1.0);
        self.compact.step(if row.page()==QuickRow::PLAY {1.0}else{0.0},dt,800.0,1.0);
        self.pause_focus.step(quick_row_top_at(row,row,hidden),dt,800.0,0.76);
        if let Some(on) = on {
            self.switch
                .step(if on { 1.0 } else { 0.0 }, dt, 800.0, 0.76);
        }
    }
    pub fn retreat_shelf(&self,canvas:crate::Canvas,draws:&mut[Draw]){
        let t=self.entry.value.clamp(0.0,1.0);let dy=t*(canvas.h+96.0);
        for draw in draws {match draw {Draw::Rect{y,..}|Draw::RoundRect{y,..}|Draw::Tex{y,..}|Draw::Part{y,..}|Draw::Turned{y,..}=>*y+=dy,_=>{}}}
    }
    pub fn reveal(&self)->f32 {self.entry.value.clamp(0.0,1.0)}
    pub fn displacement(&self, canvas: crate::Canvas) -> f32 {
        (1.0-self.entry.value)*(canvas.h+32.0)
    }

    /// The opaque sheet expands from, and contracts into, the hardware bay.
    pub fn transform(&self,canvas:crate::Canvas,draws:&mut[Draw]) {
        let reveal=self.entry.value.clamp(0.0,1.0);
        let scale=0.72+0.28*reveal;
        let anchor=canvas.h-crate::MOUTH_H;
        let dy=(1.0-reveal)*anchor*0.80;
        for draw in draws {
            if is_chrome(draw) {continue;}
            match draw {
                Draw::Rect{x,y,w,h,..}|Draw::RoundRect{x,y,w,h,..}|Draw::Tex{x,y,w,h,..}|Draw::Part{x,y,w,h,..}|Draw::Turned{x,y,w,h,..}=>{
                    *x=canvas.w/2.0+(*x-canvas.w/2.0)*scale;
                    *y=anchor+(*y-anchor)*scale+dy;*w*=scale;*h*=scale;
                },_=>{}
            }
            if let Draw::RoundRect{radius,..}=draw {*radius*=scale;}
        }
    }

    pub fn current_custom_key(&self) -> Option<u64> {
        self.custom
    }
    pub fn outgoing_custom_key(&self) -> Option<u64> { self.outgoing_page.filter(|_|self.outgoing_custom) }
    pub fn outgoing_is_custom(&self) -> bool {
        self.outgoing_custom
    }
    pub fn outgoing_row(&self) -> Option<QuickRow> {
        self.outgoing
    }
    pub fn outgoing_frame(&self, row: QuickRow, hidden: &[QuickRow]) -> MenuFrame {
        let mut frame = MenuFrame::settled(row, hidden);
        frame.page_shift = -self.direction * self.slide.value;
        frame.focus_y=self.outgoing_focus_y;
        if let Some(origin)=self.tile_origin {
            frame.tile_origin=Some(origin);
            frame.tile_reveal=1.0-self.slide.value.clamp(0.0,1.0);
        }
        if self.pause_morph {
            frame.suppress_focus = true;

        }
        frame.background = false;
        frame.outgoing = true;
        frame.opacity = 1.0;
        frame.entry = self.entry.value;
        frame
    }
    pub fn closing_row(&self) -> Option<QuickRow> {
        self.row
            .filter(|_| !self.active)
    }
    pub fn frame(&self, row: QuickRow, hidden: &[QuickRow]) -> MenuFrame {
        self.frame_page(row, hidden, None)
    }
    pub fn frame_page(&self, row: QuickRow, hidden: &[QuickRow], custom: Option<u64>) -> MenuFrame {
        if self.page != Some(custom.unwrap_or(row.page()[0].index() as u64)) {
            return MenuFrame::settled(row, hidden);
        }
        let mut frame = MenuFrame {
            grid_focus:[self.grid_x.value,self.grid_y.value],
            tile_origin:self.tile_origin.filter(|_| self.outgoing.is_some()),
            tile_reveal:self.slide.value.clamp(0.0,1.0),
            focus_y: quick_row_top_at(row, row, hidden)
                + ((self.scroll.value - target_position(row, hidden))
                    - (self.window.value - crate::quick_window(row, hidden).0 as f32))
                    * crate::QUICK_PITCH,
            scroll_position: self.scroll.value,
            window_start: self.window.value,
            page_shift: if self.active {
                self.direction * (1.0 - self.slide.value)
            } else {
                0.0
            },
            background: self.outgoing.is_none(),
            outgoing: false,
            entry: self.entry.value,
            opacity: 1.0,
            sliders: [None; 2],
            focus_compact: self.compact.value.clamp(0.0,1.0),
            suppress_focus: false,
            content_opacity: 1.0,
            switch: (self.row == Some(row)).then_some(self.switch.value.clamp(0.0, 1.0)),
            closing: !self.active,
        };

        if custom==Some(2001) {frame.focus_y+=190.0-ROW_TOP;}
        if self.pause_morph && self.outgoing.is_some() {
            frame.focus_y=self.pause_focus.value;
        }
        frame
    }
}

fn target_position(row: QuickRow, hidden: &[QuickRow]) -> f32 {
    row.position(hidden) as f32
}

/// Cached menu pages (Bluetooth and parental controls) use the same shapes.
/// Called only while rebuilding their existing text texture, never per frame.
pub fn raster_menu_rows(
    face: &mut crate::UndoFace,
    top: f32,
    pitch: f32,
    count: usize,
    selected: Option<usize>,
) {
    if let Some(i) = selected.filter(|i| *i < count) {
        raster_round(
            face,
            84.0,
            top + i as f32 * pitch,
            face.w as f32 - 168.0,
            ROW_H,
            ROW_RADIUS,
            SELECTED,
        );
    }
}

pub(crate) fn raster_round(
    face: &mut crate::UndoFace,
    left: f32,
    top: f32,
    w: f32,
    h: f32,
    r: f32,
    ink: [f32; 4],
) {
    for y in top.max(0.0) as u32..((top + h).ceil() as u32).min(face.h) {
        for x in left.max(0.0) as u32..((left + w).ceil() as u32).min(face.w) {
            let qx = ((x as f32 + 0.5 - left) - w / 2.0).abs() - w / 2.0 + r;
            let qy = ((y as f32 + 0.5 - top) - h / 2.0).abs() - h / 2.0 + r;
            let d = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() + qx.max(qy).min(0.0) - r;
            let a = (0.5 - d).clamp(0.0, 1.0) * ink[3];
            if a == 0.0 {
                continue;
            }
            let at = ((y * face.w + x) * 4) as usize;
            let old = face.rgba[at + 3] as f32 / 255.0;
            let alpha = a + old * (1.0 - a);
            for c in 0..3 {
                face.rgba[at + c] =
                    ((ink[c] * a + face.rgba[at + c] as f32 / 255.0 * old * (1.0 - a)) / alpha
                        * 255.0)
                        .round() as u8;
            }
            face.rgba[at + 3] = (alpha * 255.0).round() as u8;
        }
    }
}

#[derive(Default, Debug)]
pub struct StatusMotion {
    key: Option<u8>,
    scale: Spring,
    opacity: Spring,
}
impl StatusMotion {
    pub fn update(&mut self, key: u8, dt: f32) {
        if self.key != Some(key) {
            let initial = self.key.is_none();
            self.key = Some(key);
            self.scale = Spring::at(if initial { 1.0 } else { 0.0 });
            self.opacity = Spring::at(if initial { 1.0 } else { 0.0 });
        }
        self.scale.step(1.0, dt, 800.0, 0.8);
        self.opacity.step(1.0, dt, 1600.0, 1.0);
    }
    pub fn apply(&self, draws: &mut [Draw]) {
        let scale = 0.8 + 0.2 * self.scale.value;
        for d in draws {
            if let Draw::Tex {
                x, y, w, h, alpha, ..
            } = d
            {
                *x += *w * (1.0 - scale) / 2.0;
                *y += *h * (1.0 - scale) / 2.0;
                *w *= scale;
                *h *= scale;
                *alpha = 1.0;
            }
        }
    }
}

/// Slide an existing cached submenu texture without rerasterising text. The
/// controller legend and physical cartridge bay remain anchored.
pub fn draw_cached_menu(
    canvas: crate::Canvas,
    tex: crate::TexId,
    frame: MenuFrame,
    out: &mut Vec<Draw>,
) {
    let begin = out.len();
    let dx = frame.page_shift * (canvas.w - 2.0 * MENU_INSET);
    out.push(Draw::Part {x:dx,y:CARD_TOP,w:canvas.w,h:CARD_BOTTOM-CARD_TOP,tex,alpha:frame.opacity,
        uv:[0.0,CARD_TOP/canvas.h,1.0,(CARD_BOTTOM-CARD_TOP)/canvas.h]});
    crate::quick_menu::clip_quads(out,begin,[MENU_INSET,CARD_TOP,canvas.w-MENU_INSET,CARD_BOTTOM]);
    if !frame.outgoing {
        out.push(Draw::Part{x:0.0,y:0.0,w:canvas.w,h:TITLE_H,tex,alpha:1.0,uv:[0.0,0.0,1.0,TITLE_H/canvas.h]});
        out.push(Draw::Part{x:0.0,y:FOOTER_Y,w:canvas.w,h:44.0,tex,alpha:1.0,uv:[0.0,FOOTER_Y/canvas.h,1.0,44.0/canvas.h]});
    }
}

/// Bounded collection of independent switch springs. A new control snaps to
/// its saved state; subsequent changes retain velocity, including reversals.
#[derive(Default, Debug)]
pub struct SwitchMotion {
    states: Vec<(u64, bool, Spring)>,
}
impl SwitchMotion {
    pub fn update(&mut self, controls: &[crate::MenuSwitch], dt: f32) {
        for c in controls {
            if let Some(state) = self.states.iter_mut().find(|s| s.0 == c.id) {
                state.1 = c.on;
            } else {
                self.states
                    .push((c.id, c.on, Spring::at(if c.on { 1.0 } else { 0.0 })));
            }
        }
        for (_, on, spring) in &mut self.states {
            spring.step(if *on { 1.0 } else { 0.0 }, dt, 800.0, 0.76);
        }
    }
    pub fn value(&self, id: u64, on: bool) -> f32 {
        self.states
            .iter()
            .find(|s| s.0 == id)
            .map_or(if on { 1.0 } else { 0.0 }, |s| s.2.value.clamp(0.0, 1.0))
    }
}

#[derive(Default, Debug)]
pub struct SliderMotion {
    states: Vec<(u64, f32, Spring)>,
}
impl SliderMotion {
    pub fn update<'a>(
        &mut self,
        controls: impl IntoIterator<Item = &'a crate::MenuSlider>,
        dt: f32,
    ) {
        for c in controls {
            if let Some(s) = self.states.iter_mut().find(|s| s.0 == c.id) {
                s.1 = c.fraction;
            } else {
                self.states.push((c.id, c.fraction, Spring::at(c.fraction)));
            }
        }
        for (_, target, spring) in &mut self.states {
            spring.step(*target, dt, 1600.0, 1.0);
        }
    }
    pub fn value(&self, id: u64, fallback: f32) -> f32 {
        self.states
            .iter()
            .find(|s| s.0 == id)
            .map_or(fallback, |s| s.2.value.clamp(0.0, 1.0))
    }
}

#[derive(Default, Debug)]
pub struct RectMotion {
    x: Spring,
    y: Spring,
    ready: bool,
}
impl RectMotion {
    pub fn update(&mut self, rect: Option<[f32; 4]>, dt: f32) {
        if let Some(r) = rect {
            if !self.ready {
                self.x = Spring::at(r[0]);
                self.y = Spring::at(r[1]);
                self.ready = true;
            }
            self.x.step(r[0], dt, 800.0, 0.76);
            self.y.step(r[1], dt, 800.0, 0.76);
        } else {
            self.ready = false;
        }
    }
    pub fn frame(&self, mut r: [f32; 4]) -> [f32; 4] {
        if self.ready {
            r[0] = self.x.value;
            r[1] = self.y.value;
        }
        r
    }
}

/// A single opaque surface grows from the pause card into full Settings.
pub fn draw_pause_surface(canvas:crate::Canvas,frame:MenuFrame,hidden:&[QuickRow],out:&mut Vec<Draw>){
    let count=QuickRow::Resume.shown(hidden).len();let compact=crate::menu_grid::pause_rect(canvas,count);
    draw_pause_surface_at(canvas,frame,compact,out);
}
pub fn draw_kid_pause_surface(canvas:crate::Canvas,frame:MenuFrame,out:&mut Vec<Draw>){draw_pause_surface_at(canvas,frame,crate::menu_grid::pause_rect(canvas,3),out);}
fn draw_pause_surface_at(canvas:crate::Canvas,frame:MenuFrame,small:[f32;4],out:&mut Vec<Draw>){
    let t=frame.focus_compact.clamp(0.0,1.0);let full=card_rect(canvas);
    let a=std::array::from_fn::<_,4,_>(|i|full[i]+(small[i]-full[i])*t);
    rounded(out,a[0],a[1],a[2],a[3],28.0,SURFACE);
}
pub fn draw_shared_card(canvas:crate::Canvas,_frame:MenuFrame,_root:bool,out:&mut Vec<Draw>){draw_card(canvas,out);}


/// The hardware bay occludes moving sheets; menu ink cannot pass over it.
pub fn clip_panel(out:&mut Vec<Draw>,begin:usize,canvas:crate::Canvas) {
    crate::quick_menu::clip_quads(out,begin,[0.0,0.0,canvas.w,canvas.h-crate::MOUTH_H]);
}


pub fn draw_kid_grid_focus(canvas:crate::Canvas,frame:MenuFrame,out:&mut Vec<Draw>){
    let index=((frame.focus_y-190.0)/crate::QUICK_PITCH).clamp(0.0,2.0);
    let [x,y,w,h]=crate::menu_grid::cell(canvas,0,3,3,true);
    rounded(out,x+index*120.0+frame.page_shift*(canvas.w-112.0),y,w,h,22.0,SELECTED);
}

/// Crossfade only changing labels. Shared button legends remain fully opaque.
#[derive(Default,Debug)]
pub struct ChromeMotion {key:Option<u64>,old:Vec<Draw>,current:Vec<Draw>,progress:Spring,pending:f32}
impl ChromeMotion {
    pub fn advance(&mut self,dt:f32){self.pending+=dt.max(0.0);self.progress.step(1.0,dt,1000.0,1.0);if self.progress.value==1.0 {self.old.clear();}}
    pub fn reset(&mut self){*self=Self::default();}
    pub fn present(&mut self,key:u64,draws:Vec<Draw>,out:&mut Vec<Draw>){
        if self.current!=draws {
            let mut snapshot=vec![];self.render(&mut snapshot);
            self.old=snapshot;self.progress=Spring::at(if self.key.is_none(){1.0}else{0.0});self.key=Some(key);self.progress.step(1.0,self.pending,1000.0,1.0);
        }
        self.pending=0.0;self.current=draws;self.render(out);
    }
    fn render(&self,out:&mut Vec<Draw>){
        let t=self.progress.value.clamp(0.0,1.0);
        for d in &self.old {if !self.current.contains(d){fade(*d,1.0-t,out);}}
        for d in &self.current {fade(*d,if self.old.contains(d){1.0}else{t},out);}
    }
}
fn fade(mut d:Draw,t:f32,out:&mut Vec<Draw>){if t<=0.0{return;}match &mut d {Draw::Tex{alpha,..}|Draw::Part{alpha,..}|Draw::Turned{alpha,..}=>*alpha*=t,Draw::Rect{colour,..}|Draw::RoundRect{colour,..}=>colour[3]*=t,_=>{}}out.push(d);}
