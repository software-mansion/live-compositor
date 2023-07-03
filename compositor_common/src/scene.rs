use std::{collections::HashMap, rc::Rc};

// TODO: Put those into different modules

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VideoId(usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransformationRegistryKey(String);

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

    Transformation {
        registry_key: TransformationRegistryKey,
        inputs: HashMap<String, Rc<Node>>,
        resolution: Resolution,
    },
}

pub struct Scene {
    pub final_node: Rc<Node>,
}
