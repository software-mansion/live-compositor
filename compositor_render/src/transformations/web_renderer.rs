use std::sync::Mutex;

use crate::renderer::{
    texture::{BGRATexture, NodeTexture},
    BGRAToRGBAConverter, RegisterCtx, RenderCtx,
};

use compositor_chromium::cef;
use compositor_common::{
    renderer_spec::WebRendererSpec,
    scene::{NodeId, Resolution},
};
use log::{error, info};
use serde::{Deserialize, Serialize};

use self::{
    browser::{BrowserClient, BrowserState},
    chromium::WebRendererContextError,
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
    params: WebRendererSpec,
    state: Mutex<BrowserState>,

    bgra_texture: BGRATexture,
    _bgra_bind_group_layout: wgpu::BindGroupLayout,
    bgra_bind_group: wgpu::BindGroup,
    bgra_to_rgba: BGRAToRGBAConverter,
}

impl WebRenderer {
    pub fn new(ctx: &RegisterCtx, params: WebRendererSpec) -> Result<Self, CreateWebRendererError> {
        info!("Starting web renderer for {}", &params.url);

        let (painted_frames_sender, painted_frames_receiver) = crossbeam_channel::bounded(1);
        let state = Mutex::new(BrowserState::new(painted_frames_receiver));
        let client = BrowserClient::new(painted_frames_sender, params.resolution);
        let _browser = ctx.chromium.start_browser(&params.url, client)?;
        let _frame = _browser.main_frame().unwrap();
        let msg = cef::ProcessMessage::new("TEST");
        _frame
            .send_process_message(cef::ProcessId::Renderer, msg)
            .unwrap();

        let bgra_texture = BGRATexture::new(&ctx.wgpu_ctx, params.resolution);
        let bgra_bind_group_layout = BGRATexture::new_bind_group_layout(&ctx.wgpu_ctx.device);
        let bgra_bind_group = bgra_texture.new_bind_group(&ctx.wgpu_ctx, &bgra_bind_group_layout);
        let bgra_to_rgba = BGRAToRGBAConverter::new(&ctx.wgpu_ctx.device, &bgra_bind_group_layout);

        Ok(Self {
            params,
            state,
            bgra_texture,
            _bgra_bind_group_layout: bgra_bind_group_layout,
            bgra_bind_group,
            bgra_to_rgba,
        })
    }

    pub fn render(
        &self,
        ctx: &RenderCtx,
        _sources: &[(&NodeId, &NodeTexture)],
        target: &mut NodeTexture,
    ) {
        let mut state = self.state.lock().unwrap();
        if let Some(frame) = state.retrieve_frame() {
            let target = target.ensure_size(ctx.wgpu_ctx, self.params.resolution);
            self.bgra_texture.upload(ctx.wgpu_ctx, frame);
            self.bgra_to_rgba.convert(
                ctx.wgpu_ctx,
                (&self.bgra_texture, &self.bgra_bind_group),
                target.rgba_texture(),
            );
        }
    }

    pub fn resolution(&self) -> Resolution {
        self.params.resolution
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateWebRendererError {
    #[error("failed to create new web renderer session")]
    CreateContextFailure(#[from] WebRendererContextError),
}
