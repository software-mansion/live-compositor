use anyhow::{anyhow, Result};
use compositor_render::use_global_wgpu_ctx;
use crossbeam_channel::Sender;
use reqwest::StatusCode;
use std::{
    env,
    sync::{
        atomic::{AtomicU16, Ordering},
        OnceLock,
    },
    thread,
    time::{Duration, Instant},
};
use tracing::info;
use video_compositor::{
    config::{read_config, LoggerConfig, LoggerFormat},
    logger::{self, FfmpegLogLevel},
    server::run_api,
    state::ApiState,
};

pub struct CompositorInstance {
    pub api_port: u16,
    pub http_client: reqwest::blocking::Client,
    pub should_close_sender: Sender<()>,
}

impl Drop for CompositorInstance {
    fn drop(&mut self) {
        self.should_close_sender.send(()).unwrap();
    }
}

impl CompositorInstance {
    pub fn start() -> Self {
        init_compositor_prerequisites();
        let mut config = read_config();
        let api_port = get_free_port();
        config.api_port = api_port;

        info!("Starting LiveCompositor Integration Test with config:\n{config:#?}",);

        let (should_close_sender, should_close_receiver) = crossbeam_channel::bounded(1);
        let (state, _event_loop) = ApiState::new(config).unwrap();

        thread::Builder::new()
            .name("HTTP server startup thread".to_string())
            .spawn(move || {
                run_api(state, should_close_receiver).unwrap();
            })
            .unwrap();

        let instance = CompositorInstance {
            api_port,
            http_client: reqwest::blocking::Client::new(),
            should_close_sender,
        };
        instance.wait_for_start(Duration::from_secs(30)).unwrap();
        instance
    }

    pub fn get_port(&self) -> u16 {
        get_free_port()
    }

    pub fn send_request(&self, path: &str, request_body: serde_json::Value) -> Result<()> {
        let resp = self
            .http_client
            .post(format!("http://127.0.0.1:{}/api/{}", self.api_port, path))
            .timeout(Duration::from_secs(100))
            .json(&request_body)
            .send()?;

        if resp.status() >= StatusCode::BAD_REQUEST {
            let status = resp.status();
            let request_str = serde_json::to_string_pretty(&request_body).unwrap();
            let body_str = resp.text().unwrap();
            return Err(anyhow::anyhow!(
                "Request failed with status: {status}\nRequest: {request_str}\nResponse: {body_str}",
            ));
        }

        Ok(())
    }

    fn wait_for_start(&self, timeout: Duration) -> Result<()> {
        let start = Instant::now();
        loop {
            let response = self
                .http_client
                .get(format!("http://127.0.0.1:{}/status", self.api_port))
                .timeout(Duration::from_secs(1))
                .send();
            if response.is_ok() {
                return Ok(());
            }
            if start + timeout < Instant::now() {
                return Err(anyhow!("Failed to connect to instance."));
            }
        }
    }
}

fn get_free_port() -> u16 {
    static LAST_PORT: AtomicU16 = AtomicU16::new(10_000);
    LAST_PORT.fetch_add(1, Ordering::Relaxed)
}

fn init_compositor_prerequisites() {
    static GLOBAL_PREREQUISITES_INITIALIZED: OnceLock<()> = OnceLock::new();
    GLOBAL_PREREQUISITES_INITIALIZED.get_or_init(|| {
        env::set_var("LIVE_COMPOSITOR_WEB_RENDERER_ENABLE", "0");
        ffmpeg_next::format::network::init();
        logger::init_logger(LoggerConfig {
            ffmpeg_logger_level: FfmpegLogLevel::Info,
            format: LoggerFormat::Compact,
            level: "warn,wgpu_hal=warn,wgpu_core=warn".to_string(),
        });
        use_global_wgpu_ctx();
    });
}
