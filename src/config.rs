use std::env;
use std::fs;
use serde::{self, Serialize, Deserialize};
use crate::logging::{self, Logger};

enum ConfigError {
    CannotLoad,
    ParseError(serde_json::Error)
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "active_default")]
    pub active: bool,

    #[serde(default = "cover_art_default")]
    pub cover_art: bool
}

const fn active_default() -> bool {
    false
}

const fn cover_art_default() -> bool {
    true
}

impl Config {
    pub fn default() -> Self {
        Self {
            active: active_default(),
            cover_art: cover_art_default()
        }
    }

    pub fn from_config_file(logger: &Logger) -> Self {
        let path = Config::get_config_path();
        logging::info!(logger, "Config path {path}");
        
        match Config::parse_config_from_file(&path) {
            Ok(config) => config,
            Err(ConfigError::CannotLoad) => {
                logging::info!(logger, "Cannot load config. Using default options");
                Config::default()
            }
            Err(ConfigError::ParseError(e)) => {
                logging::error!(logger, "Cannot parse config: {e}");
                Config::default()
            }
        }
    }

    fn parse_config_from_file(path: &str) -> Result<Self, ConfigError> {
        match fs::read_to_string(path) {
            Ok(json) => Config::parse_config(&json),
            Err(_) => Err(ConfigError::CannotLoad) 
        }
    }

    fn parse_config(json: &str) -> Result<Self, ConfigError>{
        match serde_json::from_str(json) {
            Ok(config) => Ok(config),
            Err(e) => Err(ConfigError::ParseError(e))
        }
    }

    fn get_config_path() -> String {
        let mpv_home = env::var("MPV_HOME").or(
            env::var("HOME").and_then(|home| Ok(home + "/.config/mpv/")).or(
                env::var("XDG_CONFIG_HOME").and_then(|home| Ok(home + "/.mpv/"))
            )
        ).unwrap_or("/etc/mpv/".to_string());

        return mpv_home + "rpc.json"
    }
}