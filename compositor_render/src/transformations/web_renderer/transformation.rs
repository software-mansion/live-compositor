use std::{collections::HashMap, env, error::Error, num::ParseIntError, rc::Rc};

use compositor_common::scene::{Resolution, TransformationRegistryKey};
use image::ImageError;

use crate::renderer::{
    texture::Texture,
    transformation::{Transformation, TransformationParams},
    WgpuCtx,
};

use super::communication::{HttpClient, HttpClientError};

pub struct WebRenderer {
    http_client: HttpClient,
    wgpu_ctx: Rc<WgpuCtx>,
}

impl Transformation for WebRenderer {
    fn apply(
        &self,
        params: &TransformationParams,
        _sources: &HashMap<String, Texture>,
        target: &Texture,
    ) -> Result<(), Box<dyn Error>> {
        let TransformationParams::String(url) = params else {
            return Err(Box::new(WebRendererApplyError::InvalidParam));
        };

        let size = target.size();
        let frame = self.http_client.render_request(
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

impl WebRenderer {
    pub fn new(wgpu_ctx: Rc<WgpuCtx>) -> Result<Self, WebRendererNewError> {
        let port: u16 = env::var("WEB_RENDERER_PORT")?.parse()?;
        let http_client = HttpClient::new(port);

        Ok(Self {
            http_client,
            wgpu_ctx,
        })
    }

    fn upload(&self, data: &[u8], target: &Texture) -> Result<(), WebRendererApplyError> {
        let size = target.size();
        // let img = image::load_from_memory(data)?;

        // self.wgpu_ctx.queue.write_texture(
        //     wgpu::ImageCopyTexture {
        //         texture: &target.texture,
        //         mip_level: 0,
        //         origin: wgpu::Origin3d::ZERO,
        //         aspect: wgpu::TextureAspect::All,
        //     },
        //     &img.to_rgba8(),
        //     wgpu::ImageDataLayout {
        //         offset: 0,
        //         bytes_per_row: Some(4 * size.width),
        //         rows_per_image: Some(size.height),
        //     },
        //     size,
        // );

        // self.wgpu_ctx.queue.submit([]);
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WebRendererNewError {
    #[error("WEB_RENDERER_PORT env variable is not defined")]
    PortNotDefined(#[from] env::VarError),

    #[error("invalid port was provided")]
    InvalidPort(#[from] ParseIntError),
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
