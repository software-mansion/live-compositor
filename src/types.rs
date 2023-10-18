use std::fmt::Display;
use std::sync::Arc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

mod convert;
mod convert_util;
mod from_node;
mod from_renderer;
mod into_node;
mod node;
mod register_request;
mod renderer;
mod util;

#[cfg(test)]
mod convert_util_test;

pub use node::Node;
pub use node::WebRenderer;
pub use register_request::RegisterInputRequest;
pub use register_request::RegisterOutputRequest;
pub use register_request::RegisterRequest;
pub use util::Resolution;
pub use util::TypeError;

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct NodeId(Arc<str>);

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct RendererId(Arc<str>);

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct OutputId(Arc<str>);

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct InputId(Arc<str>);

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

impl Display for InputId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Display for OutputId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
