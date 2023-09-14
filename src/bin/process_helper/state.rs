use std::{collections::HashMap, sync::Mutex};

use compositor_chromium::cef;
use compositor_common::scene::NodeId;
use shared_memory::Shmem;

pub type SourceKey = (NodeId, usize);

pub struct State {
    sources: Mutex<HashMap<SourceKey, Source>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            sources: Mutex::new(HashMap::new()),
        }
    }

    pub fn insert_source(&self, key: SourceKey, shmem: Shmem, array_buffer: cef::V8Value) {
        let mut sources = self.sources.lock().unwrap();
        sources.insert(
            key,
            Source {
                _shmem: shmem,
                _array_buffer: array_buffer,
            },
        );
    }

    pub fn remove_source(&self, key: &SourceKey) {
        let mut sources = self.sources.lock().unwrap();
        sources.remove(key);
    }

    pub fn contains_source(&self, key: &SourceKey) -> bool {
        let sources = self.sources.lock().unwrap();
        sources.contains_key(key)
    }
}

struct Source {
    _shmem: Shmem,
    _array_buffer: cef::V8Value,
}
