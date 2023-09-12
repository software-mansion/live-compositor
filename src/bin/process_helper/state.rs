use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Mutex},
};

use compositor_chromium::cef;
use shared_memory::Shmem;

pub struct State {
    sources: Mutex<HashMap<String, Source>>,
    // TODO: it's debug, remove it later!
    pub f_registered: AtomicBool,
}

impl State {
    pub fn new() -> Self {
        Self {
            sources: Mutex::new(HashMap::new()),
            f_registered: AtomicBool::new(false),
        }
    }

    pub fn insert_source(&self, id: String, shmem: Shmem, array_buffer: cef::V8Value) {
        let mut sources = self.sources.lock().unwrap();
        sources.insert(
            id,
            Source {
                _shmem: shmem,
                _array_buffer: array_buffer,
            },
        );
    }

    pub fn remove_source(&self, id: &str) {
        let mut sources = self.sources.lock().unwrap();
        sources.remove(id);
    }

    pub fn contains_source(&self, id: &str) -> bool {
        let sources = self.sources.lock().unwrap();
        sources.contains_key(id)
    }
}

struct Source {
    _shmem: Shmem,
    _array_buffer: cef::V8Value,
}
