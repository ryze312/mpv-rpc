#[allow(unused_macros)]
#[macro_export]
macro_rules! error {
    ($logger:expr, $($arg:tt)*) => {
        $logger.error(&format!($($arg)*))
    };
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! warning {
    ($logger:expr, $($arg:tt)*) => {
        $logger.warning(&format!($($arg)*))
    };
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! info {
    ($logger:expr, $($arg:tt)*) => {
        $logger.info(&format!($($arg)*))
    };
}

#[allow(unused_imports)]
pub use {error, warning, info};