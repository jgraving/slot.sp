//! Offline achievement list. Cache reads and visible badge decoding stay off the UI thread.
use std::{collections::{BTreeMap,BTreeSet}, path::PathBuf, sync::mpsc};
use slot_achievements::{AchievementList, BadgeImage};
use slot_input::{Action,Btn};
use slot_ui::{Canvas,UndoFace};

type Loaded=Result<AchievementList,&'static str>;
type Images=Vec<(u32,Option<BadgeImage>)>;
pub struct Browser {
    pub data: Option<AchievementList>,
    pub message: &'static str,
    pub selected:usize,
    filter:u8,
    pub detail:Option<usize>,
    detail_max:std::cell::Cell<usize>,
    reply:Option<mpsc::Receiver<Loaded>>,
    images:Option<mpsc::Receiver<Images>>,
    badges:BTreeMap<u32,BadgeImage>,
    requested:BTreeSet<u32>,
}
impl Browser {
    pub fn open(root:PathBuf,username:String,rom:Option<PathBuf>)->Self {
        let mut view=Self::empty("Reading achievements…");
        let Some(rom)=rom else {view.message="Start a game to browse its achievements";return view;};
        let (send,receive)=mpsc::channel();
        match std::thread::Builder::new().name("slot-ra-list".into()).spawn(move||{
            let _=send.send(slot_achievements::cached_achievements(&root,&username,&rom));
        }) {Ok(_)=>view.reply=Some(receive),Err(_)=>view.message="Cannot open achievements"};
        view
    }
    pub fn empty(message:&'static str)->Self {Self{data:None,message,selected:0,filter:0,detail:None,detail_max:std::cell::Cell::new(0),reply:None,images:None,badges:BTreeMap::new(),requested:BTreeSet::new()}}
    pub fn indices(&self)->Vec<usize> {
        self.data.as_ref().map(|data|data.entries.iter().enumerate().filter(|(_,a)|
            self.filter==0 || (self.filter==1 && !a.earned) || (self.filter==2 && a.earned))
            .map(|(i,_)|i).collect()).unwrap_or_default()
    }
    pub fn scroll_info(&self)->(usize,usize,usize,usize) {
        if self.detail.is_some() {return(0,0,0,0);}
        let n=self.indices().len();let at=self.selected.min(n.saturating_sub(1));
        (at,n,at.saturating_sub(1).min(n.saturating_sub(3)),3)
    }
    pub fn update(&mut self)->bool {
        let mut changed=false;
        if let Some(reply)=&self.reply {
            match reply.try_recv() {
                Ok(result)=>{match result {Ok(data)=>{self.data=Some(data);self.message="No achievements for this game";},Err(message)=>self.message=message};self.reply=None;changed=true;},
                Err(mpsc::TryRecvError::Disconnected)=>{self.reply=None;self.message="Cannot read achievements";changed=true;},
                Err(mpsc::TryRecvError::Empty)=>{},
            }
        }
        if let Some(reply)=&self.images {
            match reply.try_recv() {
                Ok(images)=>{for (id,image) in images {if let Some(image)=image {self.badges.insert(id,image);}}self.images=None;changed=true;},
                Err(mpsc::TryRecvError::Disconnected)=>self.images=None,
                Err(mpsc::TryRecvError::Empty)=>{},
            }
        }
        // Only the three visible badges are read. Bound decoded memory for long lists.
        if self.images.is_none() {
            let indices=self.indices();let (_,_,start,visible)=self.scroll_info();
            let wanted:Vec<_>=self.data.as_ref().map(|data|indices.iter().skip(start).take(visible)
                .filter_map(|i|{let a=&data.entries[*i];a.badge.clone().map(|path|(a.id,path))})
                .filter(|(id,_)|!self.requested.contains(id)).collect()).unwrap_or_default();
            if !wanted.is_empty() {
                if self.requested.len()>24 {self.badges.clear();self.requested.clear();}
                self.requested.extend(wanted.iter().map(|(id,_)|*id));
                let (send,receive)=mpsc::channel();
                if std::thread::Builder::new().name("slot-ra-list-art".into()).spawn(move||{
                    let _=send.send(wanted.into_iter().map(|(id,path)|(id,slot_achievements::load_badge(&path))).collect());
                }).is_ok(){self.images=Some(receive);}
            }
        }
        changed
    }
    /// Returns true for Back; all other controls stay inside the browser.
    pub fn input(&mut self,action:Action)->bool {
        if let Some(scroll)=&mut self.detail {
            match action {
                Action::QuickMenu|Action::GbaDown(Btn::B)=>self.detail=None,
                Action::GbaDown(Btn::Up)=>*scroll=scroll.saturating_sub(1),
                Action::GbaDown(Btn::Down)=>*scroll=(*scroll+1).min(self.detail_max.get()),
                Action::GbaDown(Btn::L1)=>*scroll=scroll.saturating_sub(8),
                Action::GbaDown(Btn::R1)=>*scroll=(*scroll+8).min(self.detail_max.get()),
                Action::GbaDown(Btn::Left)=>{self.selected=self.selected.saturating_sub(1);*scroll=0;},
                Action::GbaDown(Btn::Right)=>{self.selected=(self.selected+1).min(self.indices().len().saturating_sub(1));self.detail=Some(0);},
                _=>{},
            }
            return false;
        }
        if matches!(action,Action::QuickMenu|Action::GbaDown(Btn::B)){return true;}
        let n=self.indices().len();
        match action {
            Action::GbaDown(Btn::A) if n>0 =>self.detail=Some(0),
            Action::GbaDown(Btn::Left|Btn::Right)=>{
                let forward=action==Action::GbaDown(Btn::Right);
                self.filter=(self.filter+if forward {1}else{2})%3;self.selected=0;
            },
            Action::GbaDown(Btn::Up)=>self.selected=self.selected.saturating_sub(1),
            Action::GbaDown(Btn::Down)=>self.selected=(self.selected+1).min(n.saturating_sub(1)),
            Action::GbaDown(Btn::L1)=>self.selected=self.selected.saturating_sub(3),
            Action::GbaDown(Btn::R1)=>self.selected=(self.selected+3).min(n.saturating_sub(1)),
            _=>{},
        }
        false
    }
    pub fn face(&self,canvas:Canvas,light:bool)->UndoFace {
        let indices=self.indices();
        if let Some(scroll)=self.detail {
            if let Some((data,entry))=self.data.as_ref().and_then(|data|indices.get(self.selected).map(|i|(data,&data.entries[*i]))) {
                self.detail_max.set(slot_ui::achievement_description_lines(&entry.description,canvas).len().saturating_sub(8));
                return slot_ui::achievement_detail_face(canvas,light,&data.title,&slot_ui::AchievementRow{title:&entry.title,points:entry.points,earned:entry.earned,badge:None},&entry.description,scroll);
            }
        }
        let (_,_,start,visible)=self.scroll_info();
        let rows:Vec<_>=self.data.as_ref().map(|data|indices.iter().skip(start).take(visible).map(|i|{
            let entry=&data.entries[*i];slot_ui::AchievementRow{
                title:&entry.title,points:entry.points,earned:entry.earned,
                badge:self.badges.get(&entry.id).map(|b|(b.rgba.as_slice(),b.width,b.height)),
            }
        }).collect()).unwrap_or_default();
        let selected=self.data.as_ref().and_then(|data|indices.get(self.selected).map(|i|&data.entries[*i]));
        let earned=self.data.as_ref().map_or(0,|d|d.entries.iter().filter(|a|a.earned).count());
        let total=self.data.as_ref().map_or(0,|d|d.entries.len());
        let filter=["All","Locked","Earned"][self.filter as usize];
        let summary=format!("{earned}/{total} earned · {filter}");
        let empty=if self.data.is_some() && !indices.is_empty(){""}else if self.data.as_ref().is_some_and(|d|!d.entries.is_empty()) {"No achievements in this filter"}else{self.message};
        slot_ui::achievement_browser_face(canvas,light,self.data.as_ref().map_or("Achievements",|d|d.title.as_str()),&summary,
            &rows,selected.map(|a|(a.title.as_str(),a.description.as_str())),empty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture()->Browser {
        let mut browser=Browser::empty("");
        browser.data=Some(AchievementList{title:"Test game".into(),entries:(0..80).map(|id|slot_achievements::AchievementEntry{
            id,title:format!("Achievement {id}"),description:"A detailed challenge".into(),points:5,earned:id%2==0,badge:None,
        }).collect()});browser
    }
    #[test]
    fn filters_paging_bounds_and_back_preserve_the_list() {
        let mut browser=fixture();assert_eq!(browser.indices().len(),80);
        browser.input(Action::GbaDown(Btn::R1));assert_eq!(browser.selected,3);assert_eq!(browser.scroll_info(),(3,80,2,3));
        browser.input(Action::GbaDown(Btn::Right));assert_eq!(browser.indices().len(),40);assert_eq!(browser.selected,0);
        assert!(browser.indices().iter().all(|i|!browser.data.as_ref().unwrap().entries[*i].earned));
        browser.input(Action::GbaDown(Btn::Right));assert!(browser.indices().iter().all(|i|browser.data.as_ref().unwrap().entries[*i].earned));
        for _ in 0..100 {browser.input(Action::GbaDown(Btn::R1));}assert_eq!(browser.selected,39);
        browser.input(Action::GbaDown(Btn::Down));assert_eq!(browser.selected,39);
        assert!(browser.input(Action::GbaDown(Btn::B)));assert_eq!(browser.data.as_ref().unwrap().entries.len(),80);
        browser.input(Action::GbaDown(Btn::Right));assert_eq!(browser.indices().len(),80);assert_eq!(browser.selected,0);
    }
    #[test]
    fn detail_scroll_reaches_full_description_and_back_restores_selection() {
        let mut browser=fixture();browser.selected=4;
        browser.data.as_mut().unwrap().entries[4].description="A long challenge with instructions that remain readable offline. ".repeat(30);
        browser.input(Action::GbaDown(Btn::A));assert_eq!(browser.detail,Some(0));
        browser.face(Canvas::FRAMED,false);assert!(browser.detail_max.get()>8);
        for _ in 0..100 {browser.input(Action::GbaDown(Btn::R1));}
        assert_eq!(browser.detail,Some(browser.detail_max.get()));
        assert!(!browser.input(Action::GbaDown(Btn::B)));assert_eq!(browser.detail,None);assert_eq!(browser.selected,4);
        assert!(browser.input(Action::GbaDown(Btn::B)));
    }
    #[test]
    fn empty_filters_and_missing_games_remain_browsable_and_closable() {
        let mut browser=Browser::open(PathBuf::new(),String::new(),None);
        browser.input(Action::GbaDown(Btn::Up));browser.input(Action::GbaDown(Btn::R1));assert_eq!(browser.selected,0);
        assert!(browser.input(Action::QuickMenu));assert_eq!(browser.message,"Start a game to browse its achievements");
        for light in [false,true] {let face=browser.face(Canvas::FRAMED,light);assert_eq!(face.rgba.len(),(face.w*face.h*4)as usize);}
    }
}
