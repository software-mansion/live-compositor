use std::{any::Any, collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VideoId(usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransformationRegistryKey(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Resolution {
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct InputId(pub String);

#[derive(Debug)]
pub enum Node {
    Video {
        id: VideoId,
        resolution: Resolution,
    },

    Image {
        data: Vec<u8>,
        resolution: Resolution,
    },

    Transformer {
        registry_key: TransformationRegistryKey,
        inputs: HashMap<InputId, Arc<Node>>,
        resolution: Resolution,
        params: Box<dyn Any>,
    },
}

pub struct Scene {
    pub final_nodes: Vec<Arc<Node>>,
}
