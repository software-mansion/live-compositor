use std::{
    cell::RefCell,
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
use log::warn;

use crate::renderer::{
    texture::Texture,
    transformation::{Transformation, TransformationParams},
    WgpuCtx,
};

use super::{
    electron_api::{ElectronApi, ElectronApiError},
    SessionId,
};

pub struct WebRenderer {
    api: ElectronApi,
    wgpu_ctx: Rc<WgpuCtx>,
    renderer_process: process::Child,
    session_ids: RefCell<HashMap<String, SessionId>>,
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

        let mut session_ids = self.session_ids.borrow_mut();

        let session_id = match session_ids.get(url) {
            Some(id) => id,
            None => {
                let size = target.size();
                let id = self.api.new_session(
                    url,
                    Resolution {
                        width: size.width as usize,
                        height: size.height as usize,
                    },
                )?;

                session_ids.insert(url.to_owned(), id);
                session_ids.get(url).unwrap()
            }
        };

        let frame = self.api.get_frame(session_id)?;
        if !frame.is_empty() {
            self.write_texture(&frame, target)?;
        }

        Ok(())
    }

    fn registry_key(&self) -> TransformationRegistryKey {
        TransformationRegistryKey("web_renderer".to_owned())
    }
}

impl Drop for WebRenderer {
    fn drop(&mut self) {
        if let Err(err) = self.renderer_process.kill() {
            warn!("Failed to stop web renderer process: {err}");
        }
    }
}

impl WebRenderer {
    pub fn new(wgpu_ctx: Rc<WgpuCtx>, port: u16) -> Result<Self, WebRendererNewError> {
        let api = ElectronApi::new(port);
        let renderer_process = Self::init_web_renderer(port)?;
        let session_ids = RefCell::new(HashMap::new());

        Ok(Self {
            api,
            wgpu_ctx,
            renderer_process,
            session_ids,
        })
    }

    fn init_web_renderer(port: u16) -> Result<process::Child, WebRendererNewError> {
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
            .args(["run", "start", "--", "--", &port.to_string()])
            .current_dir(web_renderer_path)
            .spawn()
            .map_err(WebRendererNewError::WebRendererInitError)?;

        Ok(renderer_process)
    }

    fn write_texture(&self, data: &[u8], target: &Texture) -> Result<(), WebRendererApplyError> {
        let size = target.size();
        let img = image::load_from_memory(data)?;

        if img.width() != size.width || img.height() != size.height {
            return Err(WebRendererApplyError::InvalidFrameResolution {
                expected: Resolution {
                    width: size.width as usize,
                    height: size.height as usize,
                },
                received: Resolution {
                    width: img.width() as usize,
                    height: img.height() as usize,
                },
            });
        }

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
    HttpError(#[from] ElectronApiError),

    #[error("failed to decode image data")]
    ImageDecodeError(#[from] ImageError),

    #[error("web renderer sent frame with invalid resolution. Expected {expected:?}, received {received:?}")]
    InvalidFrameResolution {
        expected: Resolution,
        received: Resolution,
    },
}
