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
        Config::get_mpv_home() + "rpc.json"
    }

    fn get_mpv_home() -> String {
        if let Ok(home) = env::var("MPV_HOME") {
            return home;
        }

        if let Ok(home) = env::var("XDG_CONFIG_HOME") {
            return home + "/mpv/";
        }

        if let Ok(home) = env::var("HOME") {
            return home + "/.config/mpv/";
        }

        "/etc/mpv/".to_owned()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            active: active_default(),
            cover_art: cover_art_default()
        }
    }
}
