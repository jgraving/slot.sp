use crate::quad::Quad;
use crate::shaders::{SPRITE_FRAG, SPRITE_VERT};
use crate::surface::{GfxError, OUT_H, OUT_W};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct TexId(usize);

#[cfg(feature = "test-support")]
impl TexId {
    pub const fn from_raw(id: usize) -> Self {
        TexId(id)
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Draw {
    Lava { w: f32, h: f32, phase: f32 },
    /// One antialiased rounded quad; no texture upload or offscreen pass.
    RoundRect {
        x: f32, y: f32, w: f32, h: f32, radius: f32, colour: [f32; 4],
    },
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        colour: [f32; 4],
    },
    Tex {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        tex: TexId,
        alpha: f32,
    },
    Part {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        tex: TexId,
        alpha: f32,
        uv: [f32; 4],
    },
    Turned {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        tex: TexId,
        alpha: f32,
        turn: f32,
    },
    Game,
    Shot {
        tex: TexId,
    },
    Capture,
}

pub struct Sprites {
    prog: gl::types::GLuint,
    white: gl::types::GLuint,
    textures: Vec<gl::types::GLuint>,
    ui_textures:std::collections::BTreeMap<usize,(u32,u32,Vec<u8>)>,
    background_textures:std::collections::BTreeMap<usize,(u32,u32,Vec<u8>)>,
    light_ui:bool,
    backdrop_textures:std::collections::BTreeMap<usize,usize>,
    backdrop_light:[bool;5],
    u_rect: gl::types::GLint,
    u_colour: gl::types::GLint,
    u_turn: gl::types::GLint,
    u_target: gl::types::GLint,
    u_src: gl::types::GLint,
    u_lava: gl::types::GLint,
    u_round: gl::types::GLint,
    u_flow: [gl::types::GLint; 6],
}

impl Sprites {
    pub fn new() -> Result<Self, GfxError> {
        let prog = crate::shaders::program(SPRITE_VERT, SPRITE_FRAG)?;
        let white = crate::gl::texture(
            1,
            1,
            gl::NEAREST,
            gl::CLAMP_TO_EDGE,
            gl::RGBA,
            Some(&[255u8; 4]),
        );
        let (u_rect, u_colour, u_turn, u_target, u_src, u_lava, u_round, u_flow);
        unsafe {
            gl::UseProgram(prog);
            gl::Uniform1i(crate::gl::uniform_location(prog, "u_tex"), 0);
            u_target = crate::gl::uniform_location(prog, "u_target");
            gl::Uniform2f(u_target, OUT_W as f32, OUT_H as f32);
            u_src = crate::gl::uniform_location(prog, "u_src");
            u_lava = crate::gl::uniform_location(prog, "u_lava");
            u_round = crate::gl::uniform_location(prog, "u_round");
            u_flow = ["u_flow0", "u_flow1", "u_flow2", "u_flow3", "u_flow4", "u_flow5"].map(|name| crate::gl::uniform_location(prog, name));
            u_rect = crate::gl::uniform_location(prog, "u_rect");
            u_colour = crate::gl::uniform_location(prog, "u_colour");
            u_turn = crate::gl::uniform_location(prog, "u_turn");
        }
        Ok(Sprites {
            prog,
            white,
            textures: Vec::new(),
            ui_textures:Default::default(),
            background_textures:Default::default(),
            light_ui:false,
            backdrop_textures:Default::default(),backdrop_light:[false;5],
            u_rect,
            u_colour,
            u_turn,
            u_target,
            u_src,
            u_lava,
            u_round,
            u_flow,
        })
    }

    pub fn set_target(&mut self, canvas: crate::Canvas) {
        unsafe {
            gl::UseProgram(self.prog);
            gl::Uniform2f(self.u_target, canvas.w, canvas.h);
        }
    }

    pub fn create_texture(&mut self, w: u32, h: u32, rgba: &[u8]) -> TexId {
        self.push(w, h, rgba, gl::LINEAR)
    }

    pub fn create_ui_texture(&mut self,w:u32,h:u32,rgba:&[u8])->TexId {
        let pixels=if self.light_ui {crate::ui_palette::light_texture(rgba)}else{rgba.to_vec()};
        let id=self.create_texture(w,h,&pixels);self.ui_textures.insert(id.0,(w,h,rgba.to_vec()));id
    }
    pub fn update_ui_texture(&mut self,id:TexId,w:u32,h:u32,rgba:&[u8]) {
        let pixels=if let Some(&zone)=self.backdrop_textures.get(&id.0) {self.backdrop_pixels(rgba,w,h,zone)}
            else if self.light_ui {crate::ui_palette::light_texture(rgba)}else{rgba.to_vec()};
        self.update_texture(id,w,h,&pixels);self.ui_textures.insert(id.0,(w,h,rgba.to_vec()));
    }
    pub fn create_background_texture(&mut self,w:u32,h:u32,rgba:&[u8])->TexId {
        let pixels=if self.light_ui {crate::ui_palette::light_background(rgba)}else{rgba.to_vec()};
        let id=self.create_texture(w,h,&pixels);self.background_textures.insert(id.0,(w,h,rgba.to_vec()));id
    }
    pub fn update_background_texture(&mut self,id:TexId,w:u32,h:u32,rgba:&[u8]) {
        let pixels=if self.light_ui {crate::ui_palette::light_background(rgba)}else{rgba.to_vec()};
        self.update_texture(id,w,h,&pixels);self.background_textures.insert(id.0,(w,h,rgba.to_vec()));
    }
    pub fn set_light_ui(&mut self,light:bool){
        if self.light_ui==light {return;}self.light_ui=light;
        let updates:Vec<_>=self.ui_textures.iter().map(|(&id,(w,h,rgba))|(TexId(id),*w,*h,if let Some(&zone)=self.backdrop_textures.get(&id) {self.backdrop_pixels(rgba,*w,*h,zone)}else if light {crate::ui_palette::light_texture(rgba)}else{rgba.clone()})).collect();
        for (id,w,h,rgba) in updates {self.update_texture(id,w,h,&rgba);}
        let backgrounds:Vec<_>=self.background_textures.iter().map(|(&id,(w,h,rgba))|(TexId(id),*w,*h,if light {crate::ui_palette::light_background(rgba)}else{rgba.clone()})).collect();
        for (id,w,h,rgba) in backgrounds {self.update_texture(id,w,h,&rgba);}
    }

    pub fn mark_backdrop_texture(&mut self,id:TexId,zone:usize) {
        assert!(zone<5);if self.backdrop_textures.get(&id.0)==Some(&zone){return;}self.backdrop_textures.insert(id.0,zone);
        if let Some((w,h,rgba))=self.ui_textures.get(&id.0).cloned(){self.update_ui_texture(id,w,h,&rgba);}
    }
    pub fn set_backdrop_light(&mut self,light:[bool;3]) {
        if self.backdrop_light[..3]==light{return;}self.backdrop_light[..3].copy_from_slice(&light);
        self.refresh_backdrop_textures();
    }
    pub fn mark_hint_texture(&mut self,id:TexId,side:usize) {
        assert!(side<2);self.mark_backdrop_texture(id,side+3);
    }
    pub fn set_hint_light(&mut self,light:[bool;2]) {
        if self.backdrop_light[3..]==light{return;}self.backdrop_light[3..].copy_from_slice(&light);
        self.refresh_backdrop_textures();
    }
    fn backdrop_pixels(&self,rgba:&[u8],w:u32,h:u32,zone:usize)->Vec<u8> {
        if zone<3 {crate::ui_palette::backdrop_text(rgba,w,h,self.backdrop_light[zone])}
        else {crate::ui_palette::hint_texture(rgba,self.backdrop_light[zone])}
    }
    fn refresh_backdrop_textures(&mut self) {
        let ids:Vec<_>=self.backdrop_textures.keys().copied().collect();
        for id in ids {if let Some((w,h,rgba))=self.ui_textures.get(&id).cloned(){self.update_ui_texture(TexId(id),w,h,&rgba);}}
    }

    pub fn create_texture_nearest(&mut self, w: u32, h: u32, rgba: &[u8]) -> TexId {
        self.push(w, h, rgba, gl::NEAREST)
    }

    fn push(&mut self, w: u32, h: u32, rgba: &[u8], filter: gl::types::GLenum) -> TexId {
        let tex = crate::gl::texture(w, h, filter, gl::CLAMP_TO_EDGE, gl::RGBA, Some(rgba));
        self.textures.push(tex);
        TexId(self.textures.len() - 1)
    }

    pub fn update_texture(&mut self, id: TexId, w: u32, h: u32, rgba: &[u8]) {
        let Some(tex) = self.textures.get(id.0) else {
            return;
        };
        if rgba.len() < (w * h * 4) as usize {
            return;
        }
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, *tex);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                crate::gl::internal_format(gl::RGBA),
                w as i32,
                h as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                rgba.as_ptr() as *const std::ffi::c_void,
            );
        }
    }

    pub fn source(&self, id: TexId) -> Option<gl::types::GLuint> {
        self.textures.get(id.0).copied()
    }

    pub fn draw(&self, items: &[Draw], quad: &Quad) {
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::UseProgram(self.prog);
            gl::ActiveTexture(gl::TEXTURE0);
        }
        // Leave ordinary carousel sprites at zero without an extra uniform
        // call per sprite. Consecutive equal-sized row shapes also share it.
        let mut previous_round = [-1.0; 3];
        for item in items {
            let whole = [0.0, 0.0, 1.0, 1.0];
            let mut flow = None;
            let mut rounded = [0.0; 3];
            let (x, y, w, h, tex, mut colour, turn, uv) = match *item {
                Draw::Lava { w, h, phase } => {
                    flow = Some(crate::lava::lava_centres(phase));
                    (0.0, 0.0, w, h, self.white, [1.0; 4], 0.0, whole)
                },
                Draw::RoundRect { x, y, w, h, radius, colour } => {
                    rounded = [w, h, radius.max(0.01).min(w.min(h) * 0.5)];
                    (x-1.0, y-1.0, w+2.0, h+2.0, self.white, colour, 0.0, whole)
                },
                Draw::Rect { x, y, w, h, colour } => (x, y, w, h, self.white, colour, 0.0, whole),
                Draw::Tex {
                    x,
                    y,
                    w,
                    h,
                    tex,
                    alpha,
                } => match self.textures.get(tex.0) {
                    Some(t) => (x, y, w, h, *t, [1.0, 1.0, 1.0, alpha], 0.0, whole),
                    None => continue,
                },
                Draw::Part {
                    x,
                    y,
                    w,
                    h,
                    tex,
                    alpha,
                    uv,
                } => match self.textures.get(tex.0) {
                    Some(t) => (x, y, w, h, *t, [1.0, 1.0, 1.0, alpha], 0.0, uv),
                    None => continue,
                },
                Draw::Turned {
                    x,
                    y,
                    w,
                    h,
                    tex,
                    alpha,
                    turn,
                } => match self.textures.get(tex.0) {
                    Some(t) => (x, y, w, h, *t, [1.0, 1.0, 1.0, alpha], turn, whole),
                    None => continue,
                },
                Draw::Game | Draw::Shot { .. } | Draw::Capture => continue,
            };
            if self.light_ui && matches!(item,Draw::RoundRect{..}) {colour=if colour[..3]==[0.0,0.0,0.0] {[0.96,0.975,0.97,colour[3]]}else{crate::ui_palette::light_colour(colour)};}
            let (cos, sin) = if turn == 0.0 {
                (1.0, 0.0)
            } else {
                (turn.cos(), turn.sin())
            };
            unsafe {
                if rounded != previous_round {
                    gl::Uniform3f(self.u_round, rounded[0], rounded[1], rounded[2]);
                    previous_round = rounded;
                }
                gl::Uniform1f(self.u_lava, if flow.is_some() { if self.light_ui { 2.0 } else { 1.0 } } else { 0.0 });
                if let Some(centres) = flow {
                    for (location, c) in self.u_flow.iter().zip(centres) {
                        gl::Uniform4f(*location, c[0], c[1], c[2], c[3]);
                    }
                }
                gl::BindTexture(gl::TEXTURE_2D, tex);
                gl::Uniform4f(self.u_rect, x, y, w, h);
                gl::Uniform2f(self.u_turn, cos, sin);
                gl::Uniform4f(self.u_src, uv[0], uv[1], uv[2], uv[3]);
                gl::Uniform4f(self.u_colour, colour[0], colour[1], colour[2], colour[3]);
            }
            quad.draw();
        }
        unsafe { gl::Disable(gl::BLEND) };
    }
}

impl Drop for Sprites {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(self.textures.len() as i32, self.textures.as_ptr());
            gl::DeleteTextures(1, &self.white);
            gl::DeleteProgram(self.prog);
        }
    }
}
