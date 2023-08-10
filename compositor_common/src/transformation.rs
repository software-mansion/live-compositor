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
    Image(ImageSpec),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebRendererTransformationParams {
    pub url: String,
    pub resolution: Resolution,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "asset_type", rename_all = "snake_case")]
pub enum ImageSpec {
    Png {
        url: String,
    },
    Jpeg {
        url: String,
    },
    Svg {
        url: String,
        resolution: Option<Resolution>,
    },
    Gif {
        url: String,
    },
}

impl ImageSpec {
    pub fn url(&self) -> &str {
        match self {
            ImageSpec::Png { url } => url,
            ImageSpec::Jpeg { url } => url,
            ImageSpec::Svg { url, .. } => url,
            ImageSpec::Gif { url } => url,
        }
    }
}
