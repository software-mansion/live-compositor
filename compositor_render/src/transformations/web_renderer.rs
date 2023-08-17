use std::sync::Mutex;

use crate::renderer::{
    texture::{BGRATexture, NodeTexture},
    RegisterTransformationCtx, RenderCtx,
};

use compositor_chromium::cef;
use compositor_common::{
    scene::{NodeId, Resolution},
    transformation::WebRendererTransformationParams,
};
use log::{error, info};
use serde::{Deserialize, Serialize};

use self::{
    browser::{BrowserClient, BrowserState},
    chromium::ChromiumContextError,
};

mod browser;
pub mod chromium;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct WebRendererOptions {
    pub init: bool,
    pub disable_gpu: bool,
}

impl Default for WebRendererOptions {
    fn default() -> Self {
        Self {
            init: true,
            disable_gpu: false,
        }
    }
}

pub struct WebRenderer {
    #[allow(dead_code)]
    params: WebRendererTransformationParams,
    // NOTE: Will be used for accessing V8 context later
    _browser: cef::Browser,
    state: Mutex<BrowserState>,
    bgra_texture: BGRATexture,
    bgra_bind_group: wgpu::BindGroup,
}

impl WebRenderer {
    pub fn new(
        ctx: &RegisterTransformationCtx,
        params: WebRendererTransformationParams,
    ) -> Result<Self, WebRendererNewError> {
        info!("Starting web renderer for {}", &params.url);

        let (painted_frames_sender, painted_frames_receiver) = crossbeam_channel::bounded(1);
        let state = Mutex::new(BrowserState::new(painted_frames_receiver));
        let client = BrowserClient::new(painted_frames_sender, params.resolution);
        let browser = ctx.chromium.start_browser(&params.url, client)?;

        let bgra_texture = BGRATexture::new(&ctx.wgpu_ctx, params.resolution);
        let bgra_bind_group =
            bgra_texture.new_bind_group(&ctx.wgpu_ctx, &ctx.wgpu_ctx.bgra_bind_group_layout);

        Ok(Self {
            params,
            _browser: browser,
            state,
            bgra_texture,
            bgra_bind_group,
        })
    }

    pub fn render(
        &self,
        ctx: &RenderCtx,
        _sources: &[(&NodeId, &NodeTexture)],
        target: &NodeTexture,
    ) {
        let mut state = self.state.lock().unwrap();
        let frame = state.retrieve_frame();

        if !frame.is_empty() {
            self.bgra_texture.upload(ctx.wgpu_ctx, frame);
            ctx.wgpu_ctx.bgra_to_rgba_converter.convert(
                ctx.wgpu_ctx,
                (&self.bgra_texture, &self.bgra_bind_group),
                &target.rgba_texture(),
            );
        }
    }

    pub fn resolution(&self) -> Resolution {
        self.params.resolution
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WebRendererNewError {
    #[error("failed to create new web renderer session")]
    CreateContextFailure(#[from] ChromiumContextError),
}
