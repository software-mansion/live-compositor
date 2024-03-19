use std::sync::{OnceLock, RwLock};

use crossbeam_channel::{unbounded, Receiver, Sender};
use log::trace;

#[derive(Debug, Clone)]
pub struct Event {
    pub kind: String,
}

pub fn emit_event<T: Into<Event>>(event: T) {
    Emitter::instance().send_event(event)
}

pub fn subscribe() -> Receiver<Event> {
    Emitter::instance().subscribe()
}

struct Emitter {
    subscribers: RwLock<Vec<Sender<Event>>>,
}

impl Emitter {
    fn instance() -> &'static Self {
        static EMITTER: OnceLock<Emitter> = OnceLock::new();
        EMITTER.get_or_init(|| Self {
            subscribers: vec![].into(),
        })
    }

    fn send_event<T: Into<Event>>(&self, event: T) {
        let event = event.into();
        for subscriber in self.subscribers.read().unwrap().iter() {
            if let Err(_err) = subscriber.send(event.clone()) {
                // TODO: remove from list
                trace!("Event subscriber disconnected.")
            }
        }
    }

    fn subscribe(&self) -> Receiver<Event> {
        let (sender, receiver) = unbounded();
        self.subscribers.write().unwrap().push(sender);
        receiver
    }
}
