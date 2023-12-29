use std::time::Duration;

use compositor_pipeline::pipeline;
use compositor_render::{scene, web_renderer};

use crate::api::{self, UpdateScene};

use super::util::*;
use super::*;

impl From<ComponentId> for scene::ComponentId {
    fn from(id: ComponentId) -> Self {
        Self(id.0)
    }
}

impl From<scene::ComponentId> for ComponentId {
    fn from(id: scene::ComponentId) -> Self {
        Self(id.0)
    }
}

impl From<RendererId> for compositor_render::RendererId {
    fn from(id: RendererId) -> Self {
        Self(id.0)
    }
}

impl From<compositor_render::RendererId> for RendererId {
    fn from(id: compositor_render::RendererId) -> Self {
        Self(id.0)
    }
}

impl From<OutputId> for compositor_render::OutputId {
    fn from(id: OutputId) -> Self {
        id.0.into()
    }
}

impl From<compositor_render::OutputId> for OutputId {
    fn from(id: compositor_render::OutputId) -> Self {
        Self(id.0)
    }
}

impl From<InputId> for compositor_render::InputId {
    fn from(id: InputId) -> Self {
        id.0.into()
    }
}

impl From<compositor_render::InputId> for InputId {
    fn from(id: compositor_render::InputId) -> Self {
        Self(id.0)
    }
}

impl TryFrom<UpdateScene> for Vec<compositor_pipeline::pipeline::OutputScene> {
    type Error = TypeError;

    fn try_from(update_scene: UpdateScene) -> Result<Self, Self::Error> {
        update_scene
            .outputs
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, TypeError>>()
    }
}

impl TryFrom<OutputScene> for compositor_pipeline::pipeline::OutputScene {
    type Error = TypeError;

    fn try_from(scene: OutputScene) -> Result<Self, Self::Error> {
        Ok(compositor_pipeline::pipeline::OutputScene {
            output_id: scene.output_id.into(),
            root: scene.root.try_into()?,
        })
    }
}

impl TryFrom<InitOptions> for pipeline::Options {
    type Error = TypeError;
    fn try_from(opts: InitOptions) -> Result<Self, Self::Error> {
        let result = Self {
            framerate: opts.framerate.try_into()?,
            stream_fallback_timeout: Duration::from_millis(
                opts.stream_fallback_timeout_ms.unwrap_or(1000.0) as u64,
            ),
            web_renderer: web_renderer::WebRendererInitOptions {
                init: opts
                    .web_renderer
                    .as_ref()
                    .and_then(|r| r.init)
                    .unwrap_or(true),
                disable_gpu: opts
                    .web_renderer
                    .as_ref()
                    .and_then(|r| r.disable_gpu)
                    .unwrap_or(false),
            },
        };
        Ok(result)
    }
}

impl TryFrom<register_request::Port> for api::Port {
    type Error = TypeError;

    fn try_from(value: register_request::Port) -> Result<Self, Self::Error> {
        const PORT_CONVERSION_ERROR_MESSAGE: &str = "Port needs to be a number between 1 and 65535 or a string in the \"START:END\" format, where START and END represent a range of ports.";
        match value {
            Port::U16(0) => Err(TypeError::new(PORT_CONVERSION_ERROR_MESSAGE)),
            Port::U16(v) => Ok(api::Port::Exact(v)),
            Port::String(s) => {
                let (start, end) = s
                    .split_once(':')
                    .ok_or(TypeError::new(PORT_CONVERSION_ERROR_MESSAGE))?;

                let start = start
                    .parse::<u16>()
                    .or(Err(TypeError::new(PORT_CONVERSION_ERROR_MESSAGE)))?;
                let end = end
                    .parse::<u16>()
                    .or(Err(TypeError::new(PORT_CONVERSION_ERROR_MESSAGE)))?;

                if start > end {
                    return Err(TypeError::new(PORT_CONVERSION_ERROR_MESSAGE));
                }

                if start == 0 || end == 0 {
                    return Err(TypeError::new(PORT_CONVERSION_ERROR_MESSAGE));
                }

                Ok(api::Port::Range((start, end)))
            }
        }
    }
}
