use std::sync::{Arc, Mutex};

use crate::renderer::{
    texture::{BGRATexture, NodeTexture},
    BGRAToRGBAConverter, RegisterCtx, RenderCtx,
};

use compositor_common::{
    renderer_spec::WebRendererSpec,
    scene::{NodeId, Resolution},
};
use log::{error, info};
use serde::{Deserialize, Serialize};

use self::{
    browser::{BrowserController, BrowserControllerNewError, EmbedFrameError},
    chromium::ChromiumContextError,
};

pub mod browser;
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
    params: WebRendererSpec,
    controller: Mutex<BrowserController>,

    bgra_texture: BGRATexture,
    _bgra_bind_group_layout: wgpu::BindGroupLayout,
    bgra_bind_group: wgpu::BindGroup,
    bgra_to_rgba: BGRAToRGBAConverter,
}

impl WebRenderer {
    pub fn new(ctx: &RegisterCtx, params: WebRendererSpec) -> Result<Self, WebRendererNewError> {
        info!("Starting web renderer for {}", &params.url);

        let bgra_texture = BGRATexture::new(&ctx.wgpu_ctx, params.resolution);
        let bgra_bind_group_layout = BGRATexture::new_bind_group_layout(&ctx.wgpu_ctx.device);
        let bgra_bind_group = bgra_texture.new_bind_group(&ctx.wgpu_ctx, &bgra_bind_group_layout);
        let bgra_to_rgba = BGRAToRGBAConverter::new(&ctx.wgpu_ctx.device, &bgra_bind_group_layout);

        let controller = Mutex::new(BrowserController::new(
            ctx.chromium.clone(),
            params.url.clone(),
            params.resolution,
        )?);

        Ok(Self {
            params,
            controller,
            bgra_texture,
            _bgra_bind_group_layout: bgra_bind_group_layout,
            bgra_bind_group,
            bgra_to_rgba,
        })
    }

    pub fn render(
        &self,
        ctx: &RenderCtx,
        sources: &[(&NodeId, &NodeTexture)],
        buffers: &[Arc<wgpu::Buffer>],
        target: &NodeTexture,
    ) -> Result<(), WebRendererRenderError> {
        let mut controller = self.controller.lock().unwrap();
        self.copy_sources_to_buffers(ctx, sources, buffers);
        controller.send_sources(ctx, sources, buffers)?;

        if let Some(frame) = controller.retrieve_frame() {
            self.bgra_texture.upload(ctx.wgpu_ctx, frame);
            self.bgra_to_rgba.convert(
                ctx.wgpu_ctx,
                (&self.bgra_texture, &self.bgra_bind_group),
                &target.rgba_texture(),
            );
        }

        Ok(())
    }

    fn copy_sources_to_buffers(
        &self,
        ctx: &RenderCtx,
        sources: &[(&NodeId, &NodeTexture)],
        buffers: &[Arc<wgpu::Buffer>],
    ) {
        let mut encoder = ctx
            .wgpu_ctx
            .device
            .create_command_encoder(&Default::default());

        for ((_id, texture), buffer) in sources.iter().zip(buffers) {
            texture.rgba_texture().copy_to_buffer(&mut encoder, buffer);
        }
        ctx.wgpu_ctx.queue.submit(Some(encoder.finish()));
    }

    pub fn resolution(&self) -> Resolution {
        self.params.resolution
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WebRendererNewError {
    #[error("Failed to create new web renderer session")]
    CreateContextFailure(#[from] ChromiumContextError),

    #[error("Failed to create browser controller")]
    CreateBrowserControllerFailure(#[from] BrowserControllerNewError),
}

#[derive(Debug, thiserror::Error)]
pub enum WebRendererRenderError {
    #[error("Failed to embed sources")]
    EmbedSources(#[from] EmbedFrameError),
}
