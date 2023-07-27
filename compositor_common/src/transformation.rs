use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::scene::Resolution;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransformationRegistryKey(pub Arc<str>);

/// TransformationSpec provides values necessary to construct entity that is used
/// internally by nodes to transform input textures.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TransformationSpec {
    Shader { source: String },
    WebRenderer(WebRendererTransformationParams),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebRendererTransformationParams {
    pub url: String,
    pub resolution: Resolution,
}
