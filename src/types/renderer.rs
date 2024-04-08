use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::util::*;

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ShaderSpec {
    /// Shader source code. [Learn more.](../../concept/shaders)
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WebRendererSpec {
    /// Url of a website that you want to render.
    pub url: String,
    /// Resolution.
    pub resolution: Resolution,
    /// Mechanism used to render input frames on the website.
    pub embedding_method: Option<WebEmbeddingMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WebEmbeddingMethod {
    /// Pass raw input frames as JS buffers so they can be rendered, for example, using a `<canvas>` component.
    ///
    /// <br/> <br/>
    ///
    /// :::warning
    ///
    /// This method might have a significant performance impact, especially for a large number of inputs.
    ///
    /// :::
    ChromiumEmbedding,

    /// Render a website without any inputs and overlay them over the website content.
    NativeEmbeddingOverContent,

    /// Render a website without any inputs and overlay them under the website content.
    NativeEmbeddingUnderContent,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(tag = "asset_type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ImageSpec {
    Png {
        url: Option<String>,
        path: Option<String>,
    },
    Jpeg {
        url: Option<String>,
        path: Option<String>,
    },
    Svg {
        url: Option<String>,
        path: Option<String>,
        resolution: Option<Resolution>,
    },
    Gif {
        url: Option<String>,
        path: Option<String>,
    },
}
