use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use slot_store::Cart;
use slot_ui::{board_face, cart_face, cart_interior, gb_shell_of, padded, CartFace, TURN_PAD};

pub struct BuiltFaces {
    pub stem: String,
    pub revision: u64,
    pub interior: CartFace,
    pub board: CartFace,
    pub lid: CartFace,
}

impl BuiltFaces {
    pub fn matches(&self,stem:&str,revision:u64)->bool {self.stem==stem && self.revision==revision}
}

pub struct FaceBuilder {
    requests: Sender<(Cart,u64)>,
    built: Receiver<BuiltFaces>,
}

impl FaceBuilder {
    pub fn spawn() -> Self {
        let (requests, inbox) = mpsc::channel::<(Cart,u64)>();
        let (outbox, built) = mpsc::channel();
        let spawned = thread::Builder::new()
            .name("slot-faces".into())
            .spawn(move || {
                while let Ok((mut cart,mut revision)) = inbox.recv() {
                    while let Ok((newer,new_revision)) = inbox.try_recv() {
                        cart = newer;revision=new_revision;
                    }
                    let faces = BuiltFaces {
                        stem: cart.stem.clone(), revision,
                        interior: if gb_shell_of(&cart).is_none() {
                            cart_interior(&cart)
                        } else {
                            CartFace {
                                rgba: vec![0; 4],
                                w: 1,
                                h: 1,
                            }
                        },
                        board: board_face(&cart),
                        lid: padded(&cart_face(&cart), TURN_PAD),
                    };
                    if outbox.send(faces).is_err() {
                        return;
                    }
                }
            });
        if let Err(e) = spawned {
            eprintln!("slot: faces: worker thread failed to start: {e}");
        }
        FaceBuilder { requests, built }
    }

    pub fn request(&self, cart: Cart) {
        self.request_revision(cart,0);
    }

    pub fn request_revision(&self, cart: Cart, revision: u64) {
        let _ = self.requests.send((cart,revision));
    }

    pub fn take(&self) -> Option<BuiltFaces> {
        let mut newest = None;
        while let Ok(faces) = self.built.try_recv() {
            newest = Some(faces);
        }
        newest
    }
}

#[cfg(test)]
mod revision_tests {
    use super::*;
    #[test]
    fn same_filename_rebuilds_and_old_revisions_cannot_match() {
        let builder=FaceBuilder::spawn();
        let mut cart=Cart{platform:slot_store::Platform::Gba,stem:"Aurora".into(),
            rom:"Games/GBA/Aurora.gba".into(),label:None,title:"AURORA".into(),code:String::new(),shell:None};
        builder.request_revision(cart.clone(),1);
        cart.shell=Some(slot_store::ShellChoice{colour:[220,40,80],
            finish:slot_store::ShellFinish::Clear,outline:slot_store::Outline::Auto});
        let expected=padded(&cart_face(&cart),TURN_PAD).rgba;
        builder.request_revision(cart,2);
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
        loop {
            assert!(std::time::Instant::now()<deadline,"revised picker artwork did not arrive");
            if let Some(faces)=builder.take() {
                if faces.matches("Aurora",2) {
                    assert!(!faces.matches("Aurora",1));assert!(!faces.matches("Other",2));
                    assert_eq!(faces.lid.rgba,expected);break;
                }
                assert_eq!(faces.revision,1);
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
}
