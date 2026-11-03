//! Adapted from BrandonKowalski/slot 4f402cb8 (GPL-3.0-or-later).
use std::sync::mpsc::{self,Receiver,TryRecvError};
use std::sync::Arc;
use slot_input::Millis;
pub const EVERY_MS:Millis=24*60*60*1000;
const RETRY_MS:Millis=60*1000;
pub type Source=Arc<dyn Fn()->Option<i64>+Send+Sync>;
pub struct NetTime {source:Source,asking:Option<Receiver<Option<i64>>>,next:Option<Millis>}
impl NetTime {
    pub fn new(source:Source)->Self {Self{source,asking:None,next:None}}
    pub fn waiting(&self)->bool {self.asking.is_some()}
    pub fn step(&mut self,ready:bool,now:Millis)->Option<i64> {
        if !ready {self.next=None;}
        if let Some(rx)=&self.asking {
            match rx.try_recv() {
                Ok(answer)=> {self.asking=None;if answer.is_none()&&ready {self.next=Some(now.saturating_add(RETRY_MS));}return answer.filter(|_|ready);}
                Err(TryRecvError::Empty)=>return None,
                Err(TryRecvError::Disconnected)=> {self.asking=None;self.next=Some(now.saturating_add(RETRY_MS));return None;}
            }
        }
        if ready&&self.next.is_none_or(|t|now>=t) {
            self.next=Some(now.saturating_add(EVERY_MS));let(tx,rx)=mpsc::channel();let source=self.source.clone();
            std::thread::spawn(move||{let _=tx.send(source());});self.asking=Some(rx);
        }
        None
    }
}
