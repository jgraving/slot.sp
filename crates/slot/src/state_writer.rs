use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use slot_store::{write_slot_state, PlayHistory, SlotState};

use crate::card_log::CardWrites;

#[derive(Default)]
struct Queue {
    next: Option<(PathBuf, SlotState)>,
    history: Option<(PathBuf, PlayHistory)>,
    busy: bool,
    closed: bool,
}

type Shared = Arc<(Mutex<Queue>, Condvar)>;

pub struct StateWriter {
    shared: Shared,
    threaded: bool,
    failing: Arc<CardWrites>,
}

impl StateWriter {
    pub fn spawn() -> Self {
        let shared: Shared = Arc::default();
        let failing: Arc<CardWrites> = Arc::default();
        let worker = shared.clone();
        let flag = failing.clone();
        let threaded = thread::Builder::new()
            .name("slot-state".into())
            .spawn(move || work(&worker, &flag))
            .is_ok();
        StateWriter {
            shared,
            threaded,
            failing,
        }
    }

    pub fn save(&self, root: &Path, state: &SlotState) {
        if !self.threaded {
            return write(root, state, &self.failing);
        }
        let (lock, wake) = &*self.shared;
        let mut q = lock.lock().unwrap_or_else(|e| e.into_inner());
        q.next = Some((root.to_path_buf(), state.clone()));
        wake.notify_all();
    }

    pub fn save_history(&self, root: &Path, history: &PlayHistory) {
        if !self.threaded { return write_history(root, history, &self.failing); }
        let (lock, wake) = &*self.shared;
        lock.lock().unwrap_or_else(|e| e.into_inner()).history = Some((root.to_path_buf(), history.clone()));
        wake.notify_all();
    }

    pub fn card_writes(&self) -> Arc<CardWrites> {
        self.failing.clone()
    }

    pub fn flush(&self) {
        let (lock, wake) = &*self.shared;
        let mut q = lock.lock().unwrap_or_else(|e| e.into_inner());
        while q.next.is_some() || q.history.is_some() || q.busy {
            q = wake.wait(q).unwrap_or_else(|e| e.into_inner());
        }
    }
}

impl Drop for StateWriter {
    fn drop(&mut self) {
        self.flush();
        let (lock, wake) = &*self.shared;
        lock.lock().unwrap_or_else(|e| e.into_inner()).closed = true;
        wake.notify_all();
    }
}

enum Job { State(PathBuf, SlotState), History(PathBuf, PlayHistory) }

fn work(shared: &Shared, failing: &CardWrites) {
    let (lock, wake) = &**shared;
    loop {
        let job = {
            let mut q = lock.lock().unwrap_or_else(|e| e.into_inner());
            loop {
                if let Some(job) = q.next.take() {
                    q.busy = true;
                    break Job::State(job.0, job.1);
                }
                if let Some((root, history)) = q.history.take() {
                    q.busy = true;
                    break Job::History(root, history);
                }
                if q.closed {
                    return;
                }
                q = wake.wait(q).unwrap_or_else(|e| e.into_inner());
            }
        };
        match job {
            Job::State(root, state) => write(&root, &state, failing),
            Job::History(root, history) => write_history(&root, &history, failing),
        }
        lock.lock().unwrap_or_else(|e| e.into_inner()).busy = false;
        wake.notify_all();
    }
}

fn write(root: &Path, state: &SlotState, failing: &CardWrites) {
    let written = write_slot_state(root, state);
    if let Err(e) = &written {
        eprintln!("slot: slot.state: {e}");
    }
    failing.wrote("slot.state", &written);
}

fn write_history(root: &Path, history: &PlayHistory, failing: &CardWrites) {
    let written = history.save(root);
    failing.wrote("play history", &written);
}
