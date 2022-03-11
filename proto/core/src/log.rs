use log::log;

use ves_proto_common::log::LogLevel;

pub struct Logger;

impl Logger {
    pub fn new() -> Self {
        Self
    }

    pub fn log(&self, level: LogLevel, msg: &str) {
        log!(
            target: concat!(env!("CARGO_CRATE_NAME"), "::game_logger"),
            level.into(),
            "{}",
            msg
        );
    }
}
