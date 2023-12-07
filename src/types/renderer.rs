use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::util::*;
use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FallbackStrategy {
    NeverFallback,
    FallbackIfAllInputsMissing,
    FallbackIfAnyInputMissing,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ShaderSpec {
    pub shader_id: RendererId,
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebRendererSpec {
    pub instance_id: RendererId,
    pub url: String,
    pub resolution: Resolution,
    pub embedding_method: Option<WebEmbeddingMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum WebEmbeddingMethod {
    ChromiumEmbedding,
    NativeEmbeddingOverContent,
    NativeEmbeddingUnderContent,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(tag = "asset_type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ImageSpec {
    Png {
        image_id: RendererId,
        url: Option<String>,
        path: Option<String>,
    },
    Jpeg {
        image_id: RendererId,
        url: Option<String>,
        path: Option<String>,
    },
    Svg {
        image_id: RendererId,
        url: Option<String>,
        path: Option<String>,
        resolution: Option<Resolution>,
    },
    Gif {
        image_id: RendererId,
        url: Option<String>,
        path: Option<String>,
    },
}
