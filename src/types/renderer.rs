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
pub struct ShaderSpec {
    /// Id of a shader. It can be used in a [`Shader`](../components/Shader) component after registration.
    pub shader_id: RendererId,
    /// Shader source code. [Learn more.](../../concept/shaders)
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct WebRendererSpec {
    /// Id of a web renderer instance. It can be used in a [`WebView`](../components/WebView) component after registration.
    pub instance_id: RendererId,
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
#[serde(tag = "asset_type", rename_all = "snake_case")]
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
