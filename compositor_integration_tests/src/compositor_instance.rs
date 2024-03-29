use anyhow::Result;
use compositor_render::use_global_wgpu_ctx;
use reqwest::StatusCode;
use std::{env, thread, time::Duration};
use video_compositor::{logger, server};

pub struct CompositorInstance {
    pub api_port: u16,
    pub http_client: reqwest::blocking::Client,
}

impl CompositorInstance {
    pub fn start(api_port: u16) -> Self {
        env::set_var("LIVE_COMPOSITOR_WEB_RENDERER_ENABLE", "0");
        ffmpeg_next::format::network::init();
        logger::init_logger();

        use_global_wgpu_ctx();

        thread::Builder::new()
            .name(format!("compositor instance on port {api_port}"))
            .spawn(move || server::run_on_port(api_port))
            .unwrap();
        thread::sleep(Duration::from_millis(5000));

        CompositorInstance {
            api_port,
            http_client: reqwest::blocking::Client::new(),
        }
    }

    pub fn send_request(&mut self, request_body: serde_json::Value) -> Result<()> {
        let resp = self
            .http_client
            .post(format!("http://127.0.0.1:{}/--/api", self.api_port))
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
}
