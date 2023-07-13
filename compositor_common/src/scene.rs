use std::{any::Any, collections::HashMap, sync::Arc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VideoId(usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransformationRegistryKey(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Resolution {
    width: usize,
    height: usize,
}

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
        inputs: HashMap<String, Arc<Node>>,
        resolution: Resolution,
        params: Box<dyn Any>,
    },
}

pub struct Scene {
    pub final_nodes: Vec<Arc<Node>>,
}
