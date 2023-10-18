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
pub struct NodeConstraints(pub Vec<Constraint>);

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Constraint {
    InputCount(InputCountConstraint),
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InputCountConstraint {
    pub fixed_count: Option<u32>,
    pub lower_bound: Option<u32>,
    pub upper_bound: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ShaderSpec {
    pub shader_id: RendererId,
    pub source: String,
    pub fallback_strategy: Option<FallbackStrategy>,
    pub constraints: Option<NodeConstraints>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebRendererSpec {
    pub instance_id: RendererId,
    pub url: String,
    pub resolution: Resolution,
    pub fallback_strategy: Option<FallbackStrategy>,
    pub constraints: Option<NodeConstraints>,
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
