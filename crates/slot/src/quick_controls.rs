use super::*;

#[derive(Default)]
pub(crate) struct QuickCard {
    pub shown: bool,
    items: Vec<usize>,
    selected: usize,
    held: Option<(Millis,bool)>,
    pub(super) motion: slot_ui::expressive::SurfaceMotion,
    icons: Vec<(TexId,u32,u32)>,
    hints: Vec<(TexId,u32,u32)>,
    pending_settings:Option<usize>,
    progress:Option<TexId>,
}
impl App {
    pub fn set_quick_control_faces(&mut self,icons:Vec<(TexId,u32,u32)>,hints:Vec<(TexId,u32,u32)>) {
        self.quick_card.icons=icons;self.quick_card.hints=hints;
    }
    pub(crate) fn set_quick_control_progress(&mut self,tex:Option<TexId>){self.quick_card.progress=tex;}
    pub fn quick_controls_open(&self)->bool {self.quick_card.shown}
    pub(super) fn quick_control_input(&mut self,action:Action)->bool {
        if action==Action::QuickControls {
            self.quick_card.pending_settings=None;
            if self.quick_card.shown {self.quick_card.shown=false;self.quick_card.held=None;}
            else if matches!(self.phase,Phase::Shelf) && self.kid_menu.is_none() && self.bluetooth.is_none() && self.core_picker.is_none() {
                self.shelf_mut().release_hold();self.play_held=None;self.held_dir=None;
                self.quick_card.items=if self.kids.active {vec![0]}else if self.bluetooth_supported {vec![0,1,2]}else{vec![0,2]};
                self.quick_card.selected=0;self.quick_card.held=None;self.quick_card.shown=true;
            }
            return true;
        }
        if self.kid_menu.is_some() {self.quick_card.shown=false;self.quick_card.held=None;self.quick_card.pending_settings=None;return false;}
        if !self.quick_card.shown {return false;}
        match action {
            Action::LidClose|Action::PowerPress|Action::PowerTap|Action::PowerHold|Action::PowerOff=>{
                self.quick_card.shown=false;self.quick_card.held=None;self.quick_card.pending_settings=None;return false;
            }
            Action::GbaDown(Btn::B)|Action::QuickMenu|Action::GbaUp(Btn::Select)=>{self.quick_card.shown=false;self.quick_card.held=None;}
            Action::GbaDown(Btn::Left|Btn::Right|Btn::Up|Btn::Down)=>{
                let n=self.quick_card.items.len();
                let forward=matches!(action,Action::GbaDown(Btn::Right|Btn::Down));
                self.quick_card.selected=(self.quick_card.selected+if forward{1}else{n-1})%n;
                self.quick_card.held=None;
            }
            Action::GbaDown(Btn::A)=>{if self.quick_card.held.is_none(){self.quick_card.held=Some((self.now(),false));}}
            Action::GbaUp(Btn::A)=>{
                if let Some((_,false))=self.quick_card.held.take() {
                    match self.quick_card.items[self.quick_card.selected] {
                        0=>self.mute_toggle(),
                        1=>if !self.quick_bluetooth.as_ref().is_some_and(|m|m.busy()) {
                            if let Some(root)=self.root.clone(){self.quick_bluetooth=Some(crate::bluetooth::Manager::quick_toggle(root));}
                        },
                        2=>self.toggle_night(),_=>{}
                    }
                }
            }
            Action::NightToggle=>self.toggle_night(),
            Action::VolumeUp|Action::VolumeDown|Action::MuteToggle|Action::BrightnessUp|Action::BrightnessDown|Action::BlueLightUp|Action::BlueLightDown=>{self.adjust(action);}
            _=>{}
        }
        true
    }
    pub(super) fn tick_quick_controls(&mut self) {
        if let Some(menu)=&mut self.quick_bluetooth {menu.poll();}
        if !matches!(self.phase,Phase::Shelf) || self.shutting_down() || self.kid_menu.is_some() {
            self.quick_card.shown=false;self.quick_card.held=None;self.quick_card.pending_settings=None;return;
        }
        if self.quick_card.pending_settings.is_none() && self.quick_card.shown && self.quick_card.held.is_some_and(|(at,fired)|!fired && self.now().saturating_sub(at)>=500) {
            self.quick_card.held=Some((self.now(),true));self.quick_card.shown=false;
            self.quick_card.pending_settings=Some(self.quick_card.items[self.quick_card.selected]);
        }
        if self.quick_card.motion.value()<=0.001 {
          if let Some(item)=self.quick_card.pending_settings.take() {
            if self.kids.active {
                self.apply(Action::QuickMenu);
                if let Some(menu)=&mut self.kid_menu {menu.input(Action::GbaDown(Btn::Down),0);}
            } else {
                let row=if item==2 {QuickRow::DisplaySchedule}else{QuickRow::Bluetooth};
                self.phase=Phase::QuickMenu{row};
                if row==QuickRow::Bluetooth && self.quick_bluetooth.as_ref().is_some_and(|m|m.busy()) {
                    self.bluetooth=self.quick_bluetooth.take();
                } else {self.open_quick_row(row);}
            }
          }
        }
    }
    pub(super) fn draw_quick_controls(&self,out:&mut Vec<Draw>)->bool {
        let amount=self.quick_card.motion.value();
        if !self.quick_card.shown && amount<=0.001 {return false;}
        // A settings page owns the canvas as soon as a hold opens it.
        if !matches!(self.phase,Phase::Shelf) || self.kid_menu.is_some() {return false;}
        self.draw_background(out);self.shelf().draw(self.canvas(),self.shelf_shake(),self.slot_faces,out);
        let begin=out.len();
        let width=self.quick_card.items.len()as f32*80.0+32.0;
        let dy=(1.0-amount)*(self.canvas().h+104.0);
        let x=self.canvas().centred(width);let y=188.0+dy;
        slot_ui::expressive::rounded(out,x-4.0,y+4.0,width+8.0,108.0,32.0,[0.0,0.0,0.0,0.18]);
        slot_ui::expressive::rounded(out,x,y,width,104.0,28.0,[0.065,0.075,0.085,1.0]);
        for (i,&item) in self.quick_card.items.iter().enumerate() {
            let cx=x+56.0+i as f32*80.0;
            let on=match item {0=>!self.muted() && self.volume()>0,1=>self.root.as_ref().is_some_and(|r|crate::bluetooth::bluetooth_enabled(r)),_=>self.night_mode()};
            if i==self.quick_card.selected {slot_ui::expressive::rounded(out,cx-35.0,y+17.0,70.0,70.0,35.0,[0.96,0.96,0.96,1.0]);}
            slot_ui::expressive::rounded(out,cx-32.0,y+20.0,64.0,64.0,32.0,if on{slot_ui::expressive::SELECTED}else{[0.16,0.18,0.20,1.0]});
            let icon=match item {0=>if on {0}else{1},1=>2,_=>3};
            if item==1 && self.quick_bluetooth.as_ref().is_some_and(|m|m.busy()) {
                if let Some(tex)=self.quick_card.progress {out.push(Draw::Tex{x:cx-20.0,y:y+32.0,w:40.0,h:40.0,tex,alpha:1.0});continue;}
            }
            if let Some(&(tex,w,h))=self.quick_card.icons.get(icon) {out.push(Draw::Tex{x:cx-w as f32/2.0,y:y+52.0-h as f32/2.0,w:w as f32,h:h as f32,tex,alpha:1.0});}
        }
        for (i,&(tex,w,h)) in self.quick_card.hints.iter().enumerate() {
            out.push(Draw::Tex{x:if i==0{16.0}else{self.canvas().w-16.0-w as f32},y:378.0+dy,w:w as f32,h:h as f32,tex,alpha:1.0});
        }
        slot_ui::expressive::clip_panel(out,begin,self.canvas());
        self.draw_status_bar(out);self.hud.draw(self.canvas(),self.now(),out);true
    }
}
