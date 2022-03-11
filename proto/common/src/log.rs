use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum LogLevel {
    /// The "error" level.
    ///
    /// Designates very serious errors.
    Error = 1,
    /// The "warn" level.
    ///
    /// Designates hazardous situations.
    Warn,
    /// The "info" level.
    ///
    /// Designates useful information.
    Info,
    /// The "debug" level.
    ///
    /// Designates lower priority information.
    Debug,
    /// The "trace" level.
    ///
    /// Designates very low priority, often extremely verbose, information.
    Trace,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        };
        f.write_str(string)
    }
}

impl TryFrom<u32> for LogLevel {
    type Error = String;
    fn try_from(val: u32) -> Result<Self, <LogLevel as TryFrom<u32>>::Error> {
        match val {
            1 => Ok(LogLevel::Error),
            2 => Ok(LogLevel::Warn),
            3 => Ok(LogLevel::Info),
            4 => Ok(LogLevel::Debug),
            5 => Ok(LogLevel::Trace),
            val => Err(format!("Invalid LogLevel value: {val}.")),
        }
    }
}

impl From<LogLevel> for u32 {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => 1,
            LogLevel::Warn => 2,
            LogLevel::Info => 3,
            LogLevel::Debug => 4,
            LogLevel::Trace => 5,
        }
    }
}

impl From<::log::Level> for LogLevel {
    fn from(level: ::log::Level) -> Self {
        match level {
            ::log::Level::Error => LogLevel::Error,
            ::log::Level::Warn => LogLevel::Warn,
            ::log::Level::Info => LogLevel::Info,
            ::log::Level::Debug => LogLevel::Debug,
            ::log::Level::Trace => LogLevel::Trace,
        }
    }
}

impl From<LogLevel> for ::log::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => ::log::Level::Error,
            LogLevel::Warn => ::log::Level::Warn,
            LogLevel::Info => ::log::Level::Info,
            LogLevel::Debug => ::log::Level::Debug,
            LogLevel::Trace => ::log::Level::Trace,
        }
    }
}
