use std::{env, path::PathBuf, str::FromStr, sync::OnceLock, time::Duration};

use compositor_pipeline::queue::QueueOptions;
use compositor_render::{web_renderer::WebRendererInitOptions, Framerate};
use log::error;

use crate::logger::FfmpegLogLevel;

#[derive(Debug)]
pub struct Config {
    pub api_port: u16,
    pub logger: LoggerConfig,
    pub stream_fallback_timeout: Duration,
    pub web_renderer: WebRendererInitOptions,
    pub force_gpu: bool,
    pub download_root: PathBuf,
    pub queue_options: QueueOptions,
    pub output_sample_rate: u32,
}

#[derive(Debug)]
pub struct LoggerConfig {
    pub ffmpeg_logger_level: FfmpegLogLevel,
    pub format: LoggerFormat,
    pub level: String,
}

#[derive(Debug, Copy, Clone)]
pub enum LoggerFormat {
    Pretty,
    Json,
    Compact,
}

impl FromStr for LoggerFormat {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(LoggerFormat::Json),
            "pretty" => Ok(LoggerFormat::Pretty),
            "compact" => Ok(LoggerFormat::Compact),
            _ => Err("invalid logger format"),
        }
    }
}

pub fn config() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();

    CONFIG.get_or_init(|| {
        read_config().expect("Failed to read the config from environment variables.")
    })
}

fn read_config() -> Result<Config, String> {
    let api_port = match env::var("LIVE_COMPOSITOR_API_PORT") {
        Ok(api_port) => api_port
            .parse::<u16>()
            .map_err(|_| "LIVE_COMPOSITOR_API_PORT has to be valid port number")?,
        Err(_) => 8081,
    };

    let ffmpeg_logger_level = match env::var("LIVE_COMPOSITOR_FFMPEG_LOGGER_LEVEL") {
        Ok(ffmpeg_log_level) => {
            FfmpegLogLevel::from_str(&ffmpeg_log_level).unwrap_or(FfmpegLogLevel::Warn)
        }
        Err(_) => FfmpegLogLevel::Warn,
    };

    let logger_level = match env::var("LIVE_COMPOSITOR_LOGGER_LEVEL") {
        Ok(level) => level,
        Err(_) => "info,wgpu_hal=warn,wgpu_core=warn".to_string(),
    };

    // When building in repo use compact logger
    let default_logger_format = match env::var("CARGO_MANIFEST_DIR") {
        Ok(_) => LoggerFormat::Compact,
        Err(_) => LoggerFormat::Json,
    };
    let logger_format = match env::var("LIVE_COMPOSITOR_LOGGER_FORMAT") {
        Ok(format) => LoggerFormat::from_str(&format).unwrap_or(default_logger_format),
        Err(_) => default_logger_format,
    };

    const DEFAULT_FRAMERATE: Framerate = Framerate { num: 30, den: 1 };
    let framerate = match env::var("LIVE_COMPOSITOR_OUTPUT_FRAMERATE") {
        Ok(framerate) => framerate_from_str(&framerate).unwrap_or(DEFAULT_FRAMERATE),
        Err(_) => DEFAULT_FRAMERATE,
    };

    const DEFAULT_WEB_RENDERER_ENABLED: bool = cfg!(feature = "web_renderer");
    let web_renderer_enable = match env::var("LIVE_COMPOSITOR_WEB_RENDERER_ENABLE") {
        Ok(enable) => bool_env_from_str(&enable).unwrap_or(DEFAULT_WEB_RENDERER_ENABLED),
        Err(_) => DEFAULT_WEB_RENDERER_ENABLED,
    };

    let web_renderer_gpu_enable = match env::var("LIVE_COMPOSITOR_WEB_RENDERER_GPU_ENABLE") {
        Ok(enable) => bool_env_from_str(&enable).unwrap_or(true),
        Err(_) => true,
    };

    let force_gpu = match env::var("LIVE_COMPOSITOR_FORCE_GPU") {
        Ok(enable) => bool_env_from_str(&enable).unwrap_or(false),
        Err(_) => false,
    };

    const DEFAULT_STREAM_FALLBACK_TIMEOUT: Duration = Duration::from_millis(500);
    let stream_fallback_timeout = match env::var("LIVE_COMPOSITOR_STREAM_FALLBACK_TIMEOUT_MS") {
        Ok(timeout_ms) => match timeout_ms.parse::<f64>() {
            Ok(timeout_ms) => Duration::from_secs_f64(timeout_ms / 1000.0),
            Err(_) => {
                error!("Invalid value provided for \"LIVE_COMPOSITOR_STREAM_FALLBACK_TIMEOUT_MS\". Falling back to default value 500ms.");
                DEFAULT_STREAM_FALLBACK_TIMEOUT
            }
        },
        Err(_) => DEFAULT_STREAM_FALLBACK_TIMEOUT,
    };

    let download_root = env::var("LIVE_COMPOSITOR_DOWNLOAD_DIR")
        .map(PathBuf::from)
        .unwrap_or(env::temp_dir());

    let ahead_of_time_processing: bool =
        match env::var("LIVE_COMPOSITOR_AHEAD_OF_TIME_PROCESSING_ENABLE") {
            Ok(enable) => bool_env_from_str(&enable).unwrap_or(false),
            Err(_) => false,
        };

    const DEFAULT_OUTPUT_SAMPLE_RATE: u32 = 48_000;
    /// Valid Opus sample rates
    const SUPPORTED_SAMPLE_RATES: [u32; 5] = [8_000, 12_000, 16_000, 24_000, 48_000];
    let output_sample_rate: u32 = match env::var("LIVE_COMPOSITOR_OUTPUT_SAMPLE_RATE") {
        Ok(sample_rate) => {
            let sample_rate = sample_rate
                .parse()
                .map_err(|_| "LIVE_COMPOSITOR_OUTPUT_SAMPLE_RATE has to be a valid number")?;

            if SUPPORTED_SAMPLE_RATES.contains(&sample_rate) {
                sample_rate
            } else {
                return Err("LIVE_COMPOSITOR_OUTPUT_SAMPLE_RATE has to be a supported sample rate. Supported sample rates are: 8000, 12000, 16000, 24000, 48000".to_string());
            }
        }
        Err(_) => DEFAULT_OUTPUT_SAMPLE_RATE,
    };

    let run_late_scheduled_events = match env::var("LIVE_COMPOSITOR_RUN_LATE_SCHEDULED_EVENTS") {
        Ok(enable) => bool_env_from_str(&enable).unwrap_or(false),
        Err(_) => false,
    };

    let config = Config {
        api_port,
        logger: LoggerConfig {
            ffmpeg_logger_level,
            format: logger_format,
            level: logger_level,
        },
        queue_options: QueueOptions {
            ahead_of_time_processing,
            output_framerate: framerate,
            output_sample_rate,
            run_late_scheduled_events,
        },
        stream_fallback_timeout,
        force_gpu,
        web_renderer: WebRendererInitOptions {
            enable: web_renderer_enable,
            enable_gpu: web_renderer_gpu_enable,
        },
        download_root,
        output_sample_rate,
    };
    Ok(config)
}

fn framerate_from_str(s: &str) -> Result<Framerate, &'static str> {
    const ERROR_MESSAGE: &str = "Framerate needs to be an unsigned integer or a string in the \"NUM/DEN\" format, where NUM and DEN are both unsigned integers.";
    if s.contains('/') {
        let Some((num_str, den_str)) = s.split_once('/') else {
            return Err(ERROR_MESSAGE);
        };
        let num = num_str.parse::<u32>().map_err(|_| ERROR_MESSAGE)?;
        let den = den_str.parse::<u32>().map_err(|_| ERROR_MESSAGE)?;
        Ok(compositor_render::Framerate { num, den })
    } else {
        Ok(compositor_render::Framerate {
            num: s.parse::<u32>().map_err(|_| ERROR_MESSAGE)?,
            den: 1,
        })
    }
}

fn bool_env_from_str(s: &str) -> Option<bool> {
    match s {
        "1" | "true" => Some(true),
        "0" | "false" => Some(false),
        _ => None,
    }
}
