use super::config::{LoggerConfig, LoggerFormat};

pub fn init_logger(opts: LoggerConfig) {
    let env_filter = tracing_subscriber::EnvFilter::new(opts.level);
    match opts.format {
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
}
