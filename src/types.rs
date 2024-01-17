use std::fmt::Display;
use std::sync::Arc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

mod component;
mod convert;
mod convert_util;
mod from_component;
mod from_renderer;
mod register_request;
mod renderer;
mod util;

#[cfg(test)]
mod convert_util_test;

pub use component::Component;
#[allow(unused_imports)]
pub use component::Image;
#[allow(unused_imports)]
pub use component::InputStream;
#[allow(unused_imports)]
pub use component::Rescaler;
#[allow(unused_imports)]
pub use component::Shader;
#[allow(unused_imports)]
pub use component::Text;
#[allow(unused_imports)]
pub use component::Tiles;
#[allow(unused_imports)]
pub use component::View;
#[allow(unused_imports)]
pub use component::WebView;

pub use register_request::RegisterOutputRequest;
pub use register_request::RegisterRequest;

#[allow(unused_imports)]
pub use renderer::ImageSpec;
#[allow(unused_imports)]
pub use renderer::ShaderSpec;
#[allow(unused_imports)]
pub use renderer::WebRendererSpec;

#[allow(unused_imports)]
pub use util::Resolution;
pub use util::TypeError;

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct ComponentId(Arc<str>);

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct RendererId(Arc<str>);

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct OutputId(Arc<str>);

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct InputId(Arc<str>);

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct OutputScene {
    pub output_id: OutputId,
    pub root: Component,
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
