use std::{env, str::FromStr};

#[derive(Debug, Clone)]
pub struct Config {
    pub api_port: u16,
    pub start_whip_whep: bool,
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

pub fn read_config() -> Config {
    try_read_config().expect("Failed to read the config from environment variables.")
}

fn try_read_config() -> Result<Config, String> {
    let api_port = match env::var("LIVE_COMPOSITOR_WHIP_WHEP_SERVER_API_PORT") {
        Ok(api_port) => api_port
            .parse::<u16>()
            .map_err(|_| "LIVE_COMPOSITOR_WHIP_WHEP_SERVER_API_PORT has to be valid port number")?,
        Err(_) => 9000,
    };

    let start_whip_whep = match env::var("LIVE_COMPOSITOR_START_WHIP_WHEP_SERVER") {
        Ok(start_whip_whep) => start_whip_whep
            .parse::<bool>()
            .map_err(|_| "LIVE_COMPOSITOR_START_WHIP_WHEP_SERVER has to be boolean value")?,
        Err(_) => true,
    };

    let config = Config {
        api_port,
        start_whip_whep,
    };
    Ok(config)
}
