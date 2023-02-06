use std::env;

pub mod macros;
pub use macros::{error, warning, info};

#[allow(dead_code)]
#[derive(PartialEq, PartialOrd)]
pub enum LogLevel {
    None = 0,
    Error = 1,
    Warn = 2,
    Info = 3
}

impl From<u32> for LogLevel {
    fn from(num: u32) -> Self {
        match num {
            0 => LogLevel::None,
            1 => LogLevel::Error,
            2 => LogLevel::Warn,
            3 => LogLevel::Info,
            _ => LogLevel::Info
        }
    }
}

#[allow(dead_code)]
pub struct Logger {
    log_level: LogLevel
}

#[allow(dead_code)]
impl Logger {
    pub fn new(log_level: LogLevel) -> Self {
        Self {
            log_level
        }
    }

    pub fn from_env() -> Self {
        let level = env::var("MPV_RPC_LOG").unwrap_or_default();
        let level: u32 = level.parse().unwrap_or(LogLevel::Error as u32);
        
        let log_level = LogLevel::from(level);
        Self {
            log_level
        }
    }

    pub fn info(&self, message: &str) {
        if self.log_level >= LogLevel::Info {
            println!("[mpv-rpc (INFO)] {}", message);
        }
    }

    pub fn warning(&self, message: &str) {
        if self.log_level >= LogLevel::Warn {
            println!("[mpv-rpc (WARN)] {}", message);
        }
    }

    pub fn error(&self, message: &str) {
        if self.log_level >= LogLevel::Error {
            println!("[mpv-rpc (ERROR)] {}", message);
        }
    }
}