use std::path::{Path, PathBuf};
use std::{collections::HashMap, sync::Mutex};

use compositor_chromium::cef;
use shared_memory::Shmem;

pub struct State {
    sources: Mutex<HashMap<PathBuf, Source>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            sources: Mutex::new(HashMap::new()),
        }
    }

    pub fn source_index(&self, key: &Path) -> Option<usize> {
        let sources = self.sources.lock().unwrap();
        sources.get(key).map(|src| src.info.source_idx)
    }

    pub fn insert_source(&self, key: PathBuf, source: Source) {
        let mut sources = self.sources.lock().unwrap();
        sources.insert(key, source);
    }

    pub fn remove_source(&self, key: &Path) {
        let mut sources = self.sources.lock().unwrap();
        sources.remove(key);
    }

    pub fn contains_source(&self, key: &Path) -> bool {
        let sources = self.sources.lock().unwrap();
        sources.contains_key(key)
    }
}

pub struct Source {
    pub _shmem: Shmem,
    pub _array_buffer: cef::V8Value,
    pub info: FrameInfo,
}

pub struct FrameInfo {
    pub source_idx: usize,
    pub width: u32,
    pub height: u32,
    pub shmem_path: PathBuf,
}
