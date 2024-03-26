use std::{
    collections::HashSet,
    mem,
    sync::{OnceLock, RwLock},
};

use crossbeam_channel::{unbounded, Receiver, Sender};
use log::trace;

#[derive(Debug, Clone)]
pub struct Event {
    pub kind: String,
    pub properties: Vec<(String, String)>,
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
        let mut disconnected_subscriber_indexes = HashSet::new();
        for (index, subscriber) in self.subscribers.read().unwrap().iter().enumerate() {
            if let Err(_err) = subscriber.send(event.clone()) {
                trace!("Event subscriber disconnected.");
                disconnected_subscriber_indexes.insert(index);
            }
        }
        if !disconnected_subscriber_indexes.is_empty() {
            self.remove_disconnected_subscribers(disconnected_subscriber_indexes)
        }
    }

    fn subscribe(&self) -> Receiver<Event> {
        let (sender, receiver) = unbounded();
        self.subscribers.write().unwrap().push(sender);
        receiver
    }

    fn remove_disconnected_subscribers(&self, disconnected: HashSet<usize>) {
        let mut guard = self.subscribers.write().unwrap();
        *guard = mem::take(&mut *guard)
            .into_iter()
            .enumerate()
            .filter_map(|(index, sender)| {
                if disconnected.contains(&index) {
                    Some(sender)
                } else {
                    None
                }
            })
            .collect()
    }
}
