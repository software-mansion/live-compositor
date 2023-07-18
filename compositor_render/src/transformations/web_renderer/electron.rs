use std::{
    env, io,
    num::ParseIntError,
    process::{self, Command},
};

use log::error;

use super::electron_api::ElectronApiClient;

pub struct Electron {
    pub(super) client: ElectronApiClient,
    renderer_process: process::Child,
}

impl Drop for Electron {
    fn drop(&mut self) {
        if let Err(err) = self.renderer_process.kill() {
            error!("Failed to stop web renderer process: {err}");
        }
    }
}

impl Electron {
    pub fn new(port: u16) -> Result<Self, ElectronNewError> {
        let api = ElectronApiClient::new(port);
        let renderer_process = Self::init_web_renderer(port)?;

        Ok(Self {
            client: api,
            renderer_process,
        })
    }

    fn init_web_renderer(port: u16) -> Result<process::Child, ElectronNewError> {
        let web_renderer_path = env::current_exe()
            .map_err(ElectronNewError::WebRendererNotFound)?
            .parent()
            .unwrap()
            .join("../../web_renderer");

        let install_exit_code = Command::new("npm")
            .arg("install")
            .current_dir(&web_renderer_path)
            .status()
            .map_err(ElectronNewError::WebRendererInitError)?;
        if !install_exit_code.success() {
            return Err(ElectronNewError::WebRendererInstallError);
        }

        let renderer_process = Command::new("npm")
            .args(["run", "start", "--", "--", &port.to_string()])
            .current_dir(web_renderer_path)
            .spawn()
            .map_err(ElectronNewError::WebRendererInitError)?;

        Ok(renderer_process)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ElectronNewError {
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
