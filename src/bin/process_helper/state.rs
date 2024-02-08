use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use compositor_chromium::cef;
use shared_memory::{Shmem, ShmemConf};

pub struct State {
    sources: HashMap<PathBuf, Source>,
}

impl State {
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }

    pub fn source(&mut self, key: &Path) -> Option<&mut Source> {
        self.sources.get_mut(key)
    }

    pub fn create_source(
        &mut self,
        frame_info: &FrameInfo,
        ctx_entered: &cef::V8ContextEntered,
    ) -> Result<&mut Source> {
        let shmem = ShmemConf::new().flink(&frame_info.shmem_path).open()?;
        let data_ptr = shmem.as_ptr();

        let id_attribute_value = cef::V8String::new(&frame_info.id_attribute).into();
        let array_buffer: cef::V8Value = unsafe {
            cef::V8ArrayBuffer::from_ptr(
                data_ptr,
                (4 * frame_info.width * frame_info.height) as usize,
                ctx_entered,
            )
        }
        .into();
        let width = cef::V8Uint::new(frame_info.width).into();
        let height = cef::V8Uint::new(frame_info.height).into();

        let source = Source {
            _shmem: shmem,
            id_attribute: frame_info.id_attribute.clone(),
            id_attribute_value,
            array_buffer,
            width,
            height,
        };

        self.sources.insert(frame_info.shmem_path.clone(), source);
        Ok(self.sources.get_mut(&frame_info.shmem_path).unwrap())
    }

    pub fn remove_source(&mut self, key: &Path) {
        self.sources.remove(key);
    }
}

pub struct Source {
    pub _shmem: Shmem,
    pub id_attribute: String,
    pub id_attribute_value: cef::V8Value,
    pub array_buffer: cef::V8Value,
    pub width: cef::V8Value,
    pub height: cef::V8Value,
}

impl Source {
    pub fn ensure(&mut self, frame_info: &FrameInfo) {
        if self.id_attribute != frame_info.id_attribute {
            let id_attribute_value = cef::V8String::new(&frame_info.id_attribute).into();
            self.id_attribute_value = id_attribute_value;
        }
    }
}

pub struct FrameInfo {
    pub source_idx: usize,
    pub width: u32,
    pub height: u32,
    pub shmem_path: PathBuf,
    pub id_attribute: String,
}
