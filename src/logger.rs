use std::{env, sync::OnceLock};

#[derive(Debug, Clone, Copy)]
enum FfmpegLogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

fn ffmpeg_log_level() -> FfmpegLogLevel {
    static LOG_LEVEL: OnceLock<FfmpegLogLevel> = OnceLock::new();

    *LOG_LEVEL.get_or_init(|| match env::var("FFMPEG_LOG_LEVEL") {
        Ok(debug) if debug == "debug" => FfmpegLogLevel::Debug,
        Ok(info) if info == "info" => FfmpegLogLevel::Info,
        Ok(warn) if warn == "warn" => FfmpegLogLevel::Warn,
        Ok(error) if error == "error" => FfmpegLogLevel::Error,
        Ok(_) => FfmpegLogLevel::Warn,
        Err(_) => FfmpegLogLevel::Warn,
    })
}

extern "C" fn ffmpeg_log_callback(
    arg1: *mut libc::c_void,
    log_level: libc::c_int,
    fmt: *const libc::c_char,
    #[cfg(not(target_arch = "aarch64"))] va_list_tag: *mut ffmpeg_next::sys::__va_list_tag,
    #[cfg(target_arch = "aarch64")] va_list_tag: ffmpeg_next::sys::va_list,
) {
    unsafe {
        match ffmpeg_log_level() {
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
    if env::var("CARGO_MANIFEST_DIR").is_ok() {
        tracing_subscriber::fmt().init();
    } else {
        tracing_subscriber::fmt().json().init();
    }
    unsafe {
        ffmpeg_next::sys::av_log_set_callback(Some(ffmpeg_log_callback));
    }
}
