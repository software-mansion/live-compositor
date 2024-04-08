use std::fmt::Display;
use std::sync::Arc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

mod audio;
mod component;
mod from_audio;
mod from_component;
mod from_register_input;
mod from_register_output;
mod from_renderer;
mod from_types;
mod from_util;
mod from_video;
mod register_input;
mod register_output;
mod renderer;
mod util;
mod video;

#[cfg(test)]
mod from_util_test;

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

#[allow(unused_imports)]
pub use register_input::Mp4;
#[allow(unused_imports)]
pub use register_output::RtpOutputStream;

#[allow(unused_imports)]
pub use register_input::RtpInputStream;

#[allow(unused_imports)]
pub use renderer::ImageSpec;
#[allow(unused_imports)]
pub use renderer::ShaderSpec;
#[allow(unused_imports)]
pub use renderer::WebRendererSpec;

#[allow(unused_imports)]
pub use util::Resolution;
pub use util::TypeError;

pub use audio::Audio;
pub use audio::MixingStrategy;

pub use video::Video;

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct ComponentId(Arc<str>);

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct RendererId(Arc<str>);

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct OutputId(Arc<str>);

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct InputId(Arc<str>);

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateOutputRequest {
    pub video: Option<Video>,
    pub audio: Option<Audio>,
    pub schedule_time_ms: Option<f64>,
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
