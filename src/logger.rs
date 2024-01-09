use std::{str::FromStr, sync::OnceLock};

use crate::config::{config, LoggerFormat};

#[derive(Debug, Clone, Copy)]
pub enum FfmpegLogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

fn ffmpeg_logger_level() -> FfmpegLogLevel {
    static LOG_LEVEL: OnceLock<FfmpegLogLevel> = OnceLock::new();

    *LOG_LEVEL.get_or_init(|| config().logger.ffmpeg_logger_level)
}

impl FromStr for FfmpegLogLevel {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "debug" => Ok(FfmpegLogLevel::Debug),
            "info" => Ok(FfmpegLogLevel::Info),
            "warn" => Ok(FfmpegLogLevel::Warn),
            "error" => Ok(FfmpegLogLevel::Error),
            _ => Err("Invalid FFmpeg logger level."),
        }
    }
}

extern "C" fn ffmpeg_log_callback(
    arg1: *mut libc::c_void,
    log_level: libc::c_int,
    fmt: *const libc::c_char,
    #[cfg(not(target_arch = "aarch64"))] va_list_tag: *mut ffmpeg_next::sys::__va_list_tag,
    #[cfg(target_arch = "aarch64")] va_list_tag: ffmpeg_next::sys::va_list,
) {
    unsafe {
        match ffmpeg_logger_level() {
            FfmpegLogLevel::Error if log_level <= 16 => {
                ffmpeg_next::sys::av_log_default_callback(arg1, log_level, fmt, va_list_tag)
            }
            FfmpegLogLevel::Warn if log_level <= 24 => {
                ffmpeg_next::sys::av_log_default_callback(arg1, log_level, fmt, va_list_tag)
            }
            FfmpegLogLevel::Info if log_level <= 32 => {
                ffmpeg_next::sys::av_log_default_callback(arg1, log_level, fmt, va_list_tag)
            }
            FfmpegLogLevel::Debug if log_level <= 48 => {
                ffmpeg_next::sys::av_log_default_callback(arg1, log_level, fmt, va_list_tag)
            }
            _ => (),
        }
    }
}

pub fn init_logger() {
    let env_filter = tracing_subscriber::EnvFilter::new(&config().logger.level);
    match config().logger.format {
        LoggerFormat::Pretty => {
            tracing_subscriber::fmt()
                .pretty()
                .with_env_filter(env_filter)
                .init();
        }
        LoggerFormat::Json => {
            tracing_subscriber::fmt()
                .json()
                .with_env_filter(env_filter)
                .init();
        }
        LoggerFormat::Compact => {
            tracing_subscriber::fmt()
                .compact()
                .with_env_filter(env_filter)
                .init();
        }
    }
    unsafe {
        ffmpeg_next::sys::av_log_set_callback(Some(ffmpeg_log_callback));
    }
}
