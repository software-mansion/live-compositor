use crate::UNEMBED_SOURCE_MESSAGE;
use compositor_chromium::cef;
use compositor_common::scene::NodeId;
use log::error;
use shared_memory::{Shmem, ShmemConf, ShmemError};
use std::path::{Path, PathBuf};
use std::{fs, io};

pub struct SharedMemory {
    state: SharedMemoryState,
    path: PathBuf,
    node_id: NodeId,
    source_idx: usize,
}

impl SharedMemory {
    pub fn new(
        root_path: &Path,
        node_id: &NodeId,
        source_idx: usize,
        size: usize,
    ) -> Result<Self, SharedMemoryError> {
        let path = root_path.join(node_id.to_string());
        Self::init_shared_memory_folder(&path)?;

        let shmem_path = path.join(source_idx.to_string());
        let mut shared_memory = Self {
            state: SharedMemoryState::Unloaded,
            path: shmem_path,
            node_id: node_id.clone(),
            source_idx,
        };
        shared_memory.init_shared_memory(size)?;

        Ok(shared_memory)
    }

    fn init_shared_memory(&mut self, size: usize) -> Result<(), SharedMemoryError> {
        let inner = ShmemConf::new()
            .flink(&self.path)
            .size(size)
            .force_create_flink()
            .create()?;
        self.state = SharedMemoryState::Loaded(inner);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.state
            .shared_memory()
            .map(Shmem::len)
            .unwrap_or_default()
    }

    pub fn to_path_string(&self) -> String {
        self.path.display().to_string()
    }

    pub fn request_resize(
        &mut self,
        size: usize,
        frame: &cef::Frame,
    ) -> Result<(), SharedMemoryError> {
        if let SharedMemoryState::PendingResize { new_size, .. } = &mut self.state {
            *new_size = size;
            return Ok(());
        }

        let shared_memory = match self.state.unload() {
            SharedMemoryState::Loaded(shared_memory)
            | SharedMemoryState::PendingResize { shared_memory, .. } => shared_memory,
            SharedMemoryState::Unloaded => return Err(SharedMemoryError::SharedMemoryInaccessible),
        };
        self.state = SharedMemoryState::PendingResize {
            shared_memory,
            new_size: size,
        };

        let mut msg = cef::ProcessMessage::new(UNEMBED_SOURCE_MESSAGE);
        msg.write_string(0, self.to_path_string());
        msg.write_string(1, self.node_id.to_string());
        msg.write_int(2, self.source_idx as i32);
        frame.send_process_message(cef::ProcessId::Renderer, msg)?;

        Ok(())
    }

    pub fn resolve_pending_resize(&mut self) -> Result<(), SharedMemoryError> {
        let SharedMemoryState::PendingResize { new_size, .. } = self.state else {
            return Err(SharedMemoryError::PendingResizeNotFound);
        };
        // Releasing the current `Shmem` instance to ensure it does not erase the shared memory descriptor from the file system
        // This is critical to ensure when a new `Shmem` is created at the same location, it doesn't conflict with the old descriptor
        let old_state = self.state.unload();
        drop(old_state);

        // After releasing the old `Shmem`, establish a new one from the existing path with the updated size
        self.init_shared_memory(new_size)?;

        Ok(())
    }

    pub fn is_accessible(&self) -> bool {
        matches!(self.state, SharedMemoryState::Loaded(_))
    }

    pub fn write(&mut self, data: &[u8], offset: usize) -> Result<(), SharedMemoryError> {
        let Some(inner) = self.state.shared_memory() else {
            return Err(SharedMemoryError::SharedMemoryInaccessible);
        };

        if inner.len() < offset + data.len() {
            return Err(SharedMemoryError::OutOfBounds {
                shared_memory_len: inner.len(),
                write_len: offset + data.len(),
            });
        }

        let shmem = inner.as_ptr();
        unsafe {
            std::ptr::copy(data.as_ptr(), shmem.add(offset), data.len());
        }

        Ok(())
    }

    fn init_shared_memory_folder(root_shmem_folder: &Path) -> Result<(), SharedMemoryError> {
        if root_shmem_folder.exists() {
            return Ok(());
        }

        fs::create_dir_all(root_shmem_folder).map_err(SharedMemoryError::CreateShmemFolderFailed)
    }
}

enum SharedMemoryState {
    Unloaded,
    Loaded(Shmem),
    PendingResize {
        shared_memory: Shmem,
        new_size: usize,
    },
}

impl SharedMemoryState {
    fn shared_memory(&self) -> Option<&Shmem> {
        match self {
            SharedMemoryState::Unloaded | SharedMemoryState::PendingResize { .. } => None,
            SharedMemoryState::Loaded(shared_memory) => Some(shared_memory),
        }
    }

    fn unload(&mut self) -> Self {
        std::mem::replace(self, Self::Unloaded)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SharedMemoryError {
    #[error("Failed to create shared memory")]
    CreateSharedMemoryFailed(#[from] ShmemError),

    #[error(
        "Tried to write outside of the shared memory bounds: {shared_memory_len} < {write_len}"
    )]
    OutOfBounds {
        shared_memory_len: usize,
        write_len: usize,
    },

    #[error("Browser frame is no longer alive")]
    FrameNotAlive(#[from] cef::FrameError),

    #[error("Failed to create folder for shared memory")]
    CreateShmemFolderFailed(#[source] io::Error),

    #[error("There is no pending shared memory resize")]
    PendingResizeNotFound,

    #[error("Shared memory is currently inaccessible")]
    SharedMemoryInaccessible,
}
