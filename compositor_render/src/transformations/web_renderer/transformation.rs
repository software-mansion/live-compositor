use std::{
    collections::HashMap,
    env,
    error::Error,
    io,
    num::ParseIntError,
    process::{self, Command},
    rc::Rc,
};

use compositor_common::scene::{NodeId, Resolution, TransformationRegistryKey};
use image::ImageError;

use crate::renderer::{
    texture::Texture,
    transformation::{Transformation, TransformationParams},
    WgpuCtx,
};

use super::http_client::{HttpClient, HttpClientError};

pub struct WebRenderer {
    http_client: HttpClient,
    wgpu_ctx: Rc<WgpuCtx>,
    renderer_process: process::Child,
}

impl Transformation for WebRenderer {
    fn apply(
        &self,
        params: &TransformationParams,
        _sources: &HashMap<NodeId, Texture>,
        target: &Texture,
    ) -> Result<(), Box<dyn Error>> {
        let TransformationParams::String(url) = params else {
            return Err(Box::new(WebRendererApplyError::InvalidParam));
        };

        let size = target.size();
        let frame = self.http_client.get_frame(
            url.to_owned(),
            Resolution {
                width: size.width as usize,
                height: size.height as usize,
            },
        )?;

        if !frame.is_empty() {
            self.upload(&frame, target)?;
        }

        Ok(())
    }

    fn registry_key(&self) -> TransformationRegistryKey {
        TransformationRegistryKey("web_renderer".to_owned())
    }
}

impl Drop for WebRenderer {
    fn drop(&mut self) {
        self.renderer_process.kill().unwrap();
    }
}

impl WebRenderer {
    pub fn new(wgpu_ctx: Rc<WgpuCtx>) -> Result<Self, WebRendererNewError> {
        let port: u16 = env::var("WEB_RENDERER_PORT")?.parse()?;
        let http_client = HttpClient::new(port);
        let renderer_process = Self::init_web_renderer()?;

        Ok(Self {
            http_client,
            wgpu_ctx,
            renderer_process,
        })
    }

    fn init_web_renderer() -> Result<process::Child, WebRendererNewError> {
        let web_renderer_path = env::current_exe()
            .map_err(WebRendererNewError::WebRendererNotFound)?
            .parent()
            .unwrap()
            .join("../../web_renderer");

        let install_exit_code = Command::new("npm")
            .arg("install")
            .current_dir(&web_renderer_path)
            .status()
            .map_err(WebRendererNewError::WebRendererInitError)?;
        if !install_exit_code.success() {
            return Err(WebRendererNewError::WebRendererInstallError);
        }

        let renderer_process = Command::new("npm")
            .args(["run", "start"])
            .current_dir(web_renderer_path)
            .spawn()
            .map_err(WebRendererNewError::WebRendererInitError)?;

        Ok(renderer_process)
    }

    fn upload(&self, data: &[u8], target: &Texture) -> Result<(), WebRendererApplyError> {
        let size = target.size();
        let img = image::load_from_memory(data)?;

        self.wgpu_ctx.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &target.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &img.to_rgba8(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.width),
                rows_per_image: Some(size.height),
            },
            size,
        );

        self.wgpu_ctx.queue.submit([]);
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WebRendererNewError {
    #[error("WEB_RENDERER_PORT env variable is not defined")]
    PortNotDefined(#[from] env::VarError),

    #[error("invalid port was provided")]
    InvalidPort(#[from] ParseIntError),

    #[error("failed to find web renderer")]
    WebRendererNotFound(io::Error),

    #[error("failed to install web renderer deps")]
    WebRendererInstallError,

    #[error("failed to find web renderer")]
    WebRendererInitError(io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum WebRendererApplyError {
    #[error("expected string param")]
    InvalidParam,

    #[error("communication with web renderer failed")]
    HttpError(#[from] HttpClientError),

    #[error("failed to decode image data")]
    ImageDecodeError(#[from] ImageError),
}
