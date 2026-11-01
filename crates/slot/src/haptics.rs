//! Short, rate-limited feedback. The existing game rumble remains independent.
use slot_input::Millis;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Kind { Ui, Charging }
#[derive(Default)]
pub struct Feedback { active:Option<(Kind,Millis)>, last_ui:Option<Millis> }
impl Feedback {
    pub fn request(&mut self,kind:Kind,now:Millis){
        if kind==Kind::Ui {
            if self.last_ui.is_some_and(|at|now.saturating_sub(at)<85) || self.active.is_some_and(|(k,at)|k==Kind::Charging&&now.saturating_sub(at)<170){return;}
            self.last_ui=Some(now);
        }
        self.active=Some((kind,now));
    }
    pub fn remaining(&self,now:Millis)->u16 {
        self.active.map_or(0,|(kind,at)|{let age=now.saturating_sub(at);let end:u64=match kind {Kind::Ui=>8,Kind::Charging if age<35=>35,Kind::Charging=>145};end.saturating_sub(age).min(u16::MAX as u64)as u16})
    }
    pub fn clear(&mut self){self.active=None;}
    pub fn strength(&mut self,now:Millis,ui:bool,charging:bool)->u16 {
        let Some((kind,at))=self.active else{return 0;};let age=now.saturating_sub(at);
        let (enabled,duration,on)=match kind {
            Kind::Ui=>(ui,8,age<8),
            Kind::Charging=>(charging,145,age<35||(105..145).contains(&age)),
        };
        if !enabled||age>=duration {self.clear();return 0;}
        if on {match kind {Kind::Ui=>u16::MAX/8,Kind::Charging=>u16::MAX/5}}else{0}
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn ui_ticks_do_not_become_a_continuous_buzz(){let mut f=Feedback::default();f.request(Kind::Ui,0);assert!(f.strength(0,true,true)>0);f.request(Kind::Ui,5);assert_eq!(f.strength(9,true,true),0);f.request(Kind::Ui,90);assert!(f.strength(90,true,true)>0);assert_eq!(f.strength(91,false,true),0);}
    #[test] fn charging_is_two_bounded_pulses_and_is_not_restarted_by_navigation(){let mut f=Feedback::default();f.request(Kind::Charging,100);assert!(f.strength(100,true,true)>0);assert_eq!(f.strength(160,true,true),0);f.request(Kind::Ui,170);assert_eq!(f.strength(170,true,true),0);assert!(f.strength(210,true,true)>0);assert_eq!(f.strength(270,true,true),0);}
    #[test] fn disabling_or_sleeping_stops_an_active_pattern(){let mut f=Feedback::default();f.request(Kind::Charging,0);assert_eq!(f.strength(0,true,false),0);f.request(Kind::Ui,100);f.clear();assert_eq!(f.strength(101,true,true),0);}
}
