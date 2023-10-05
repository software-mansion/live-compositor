use std::sync::Arc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

mod convert;
mod convert_util;
mod from_node;
mod into_node;
mod node;
mod util;

pub use node::Node;
pub use node::WebRenderer;

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct NodeId(Arc<str>);

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct RendererId(Arc<str>);

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct OutputId(Arc<str>);

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct Resolution {
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct Scene {
    pub nodes: Vec<Node>,
    pub outputs: Vec<Output>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct Output {
    pub output_id: OutputId,
    pub input_pad: NodeId,
}
