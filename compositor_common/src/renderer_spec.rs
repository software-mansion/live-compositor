use std::{fmt::Display, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::scene::{
    validation::constraints::{input_count::InputsCountConstraint, Constraints},
    Resolution,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RendererId(pub Arc<str>);

impl Display for RendererId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum FallbackStrategy {
    NeverFallback,
    FallbackIfAllInputsMissing,
    FallbackIfAnyInputMissing,
}

/// RendererSpec provides configuration necessary to construct Renderer. Renderers
/// are entities like shader, image or chromium_instance and can be used by nodes
/// to transform or generate frames.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RendererSpec {
    Shader(ShaderSpec),
    WebRenderer(WebRendererSpec),
    Image(ImageSpec),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShaderSpec {
    pub shader_id: RendererId,
    pub source: String,
    #[serde(default = "ShaderSpec::default_fallback")]
    pub fallback_strategy: FallbackStrategy,
    #[serde(default = "ShaderSpec::default_constraints")]
    pub constraints: Constraints,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebRendererSpec {
    pub instance_id: RendererId,
    pub url: String,
    pub resolution: Resolution,
    #[serde(default = "WebRendererSpec::default_fallback")]
    pub fallback_strategy: FallbackStrategy,
    #[serde(default = "WebRendererSpec::default_constraints")]
    pub constraints: Constraints,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde()]
pub struct ImageSpec {
    #[serde(flatten)]
    pub src: ImageSrc,
    pub image_id: RendererId,

    #[serde(flatten)]
    pub image_type: ImageType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ImageSrc {
    Url { url: String },
    LocalPath { path: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "asset_type", rename_all = "snake_case")]
pub enum ImageType {
    Png,
    Jpeg,
    Svg { resolution: Option<Resolution> },
    Gif,
}

impl ShaderSpec {
    fn default_fallback() -> FallbackStrategy {
        FallbackStrategy::FallbackIfAllInputsMissing
    }

    fn default_constraints() -> Constraints {
        Constraints {
            inputs_count: InputsCountConstraint::Bounded {
                minimal: 0,
                maximal: 16,
            },
        }
    }
}

impl WebRendererSpec {
    fn default_fallback() -> FallbackStrategy {
        FallbackStrategy::FallbackIfAllInputsMissing
    }

    fn default_constraints() -> Constraints {
        Constraints {
            inputs_count: InputsCountConstraint::Bounded {
                minimal: 0,
                maximal: 16,
            },
        }
    }
}
