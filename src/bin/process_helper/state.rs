use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use compositor_chromium::cef;
use shared_memory::{Shmem, ShmemConf};

pub struct State {
    input_mappings: Vec<Arc<str>>,
    sources: HashMap<PathBuf, Arc<Source>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            input_mappings: Vec::new(),
            sources: HashMap::new(),
        }
    }

    pub fn source(&self, key: &Path) -> Option<Arc<Source>> {
        self.sources.get(key).cloned()
    }

    pub fn create_source(
        &mut self,
        frame_info: FrameInfo,
        ctx_entered: &cef::V8ContextEntered,
    ) -> Result<Arc<Source>> {
        let source_id = self.input_name(frame_info.source_idx)?;
        let shmem = ShmemConf::new().flink(&frame_info.shmem_path).open()?;
        let data_ptr = shmem.as_ptr();

        let source_id = cef::V8String::new(&source_id).into();
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

        // `Arc` is used instead of `Rc` because we can't make any guarantees that Chromium will run this code on a single thread.
        #[allow(clippy::arc_with_non_send_sync)]
        let source = Arc::new(Source {
            _shmem: shmem,
            source_index: frame_info.source_idx,
            source_id,
            array_buffer,
            width,
            height,
        });

        self.sources.insert(frame_info.shmem_path, source.clone());
        Ok(source)
    }

    pub fn remove_source(&mut self, key: &Path) {
        self.sources.remove(key);
    }

    pub fn set_input_mappings(&mut self, new_input_mappings: Vec<Arc<str>>) {
        self.sources.clear();
        self.input_mappings = new_input_mappings;
    }

    pub fn input_name(&self, source_idx: usize) -> Result<Arc<str>> {
        self.input_mappings.get(source_idx).cloned().with_context(|| {
            format!("Could not retrieve input name. Expected registered input for index {source_idx}. Use register_inputs(\"input_name1\", ...)")
        })
    }
}

pub struct Source {
    pub _shmem: Shmem,
    pub source_index: usize,
    pub source_id: cef::V8Value,
    pub array_buffer: cef::V8Value,
    pub width: cef::V8Value,
    pub height: cef::V8Value,
}

pub struct FrameInfo {
    pub source_idx: usize,
    pub width: u32,
    pub height: u32,
    pub shmem_path: PathBuf,
}
