use std::sync::Mutex;

use crate::renderer::{
    texture::{BGRATexture, NodeTexture},
    RegisterTransformationCtx, RenderCtx,
};

mod browser;
pub mod chromium;

use compositor_chromium::cef;
use compositor_common::{
    scene::{NodeId, Resolution},
    transformation::WebRendererTransformationParams,
};
use image::ImageError;
use log::{error, info};
use serde::{Deserialize, Serialize};

use self::{
    browser::{BrowserClient, BrowserState},
    chromium::ChromiumContextError,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionId(i32);

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

        let (frame_tx, frame_rx) = crossbeam_channel::bounded(1);
        let state = Mutex::new(BrowserState::new(frame_rx));
        let client = BrowserClient::new(frame_tx, params.resolution);
        let browser = ctx.chromium.start_browser(&params.url, client)?;

        let bgra_texture = BGRATexture::new(&ctx.wgpu_ctx, params.resolution);
        let bgra_bind_group = ctx
            .wgpu_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Web renderer BGRA texture bind group"),
                layout: &ctx.wgpu_ctx.rgba_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&bgra_texture.texture().view),
                }],
            });

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
    ) -> Result<(), WebRendererRenderError> {
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

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WebRendererNewError {
    #[error("failed to create new web renderer session")]
    SessionCreationError(#[from] ChromiumContextError),

    #[error("failed to retrieve browser ID")]
    GetBrowserIdFailure(#[from] cef::BrowserError),
}

#[derive(Debug, thiserror::Error)]
pub enum WebRendererRenderError {
    #[error("expected string param")]
    InvalidParam,

    #[error("failed to decode image data")]
    ImageDecodeError(#[from] ImageError),

    #[error("web renderer sent frame with invalid resolution. Expected {expected:?}, received {received:?}")]
    InvalidFrameResolution {
        expected: Resolution,
        received: Resolution,
    },
}
