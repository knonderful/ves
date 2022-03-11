use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};

pub struct Logger {
    log_fn: unsafe extern "C" fn(u32, *const u8, usize),
}

impl Logger {
    /// Creates a new instance.
    ///
    /// Note that `init()` must be called for the logger to actually take effect.
    ///
    /// # Arguments
    ///
    /// * `log_fn`: The function pointer for logging to the Core.
    #[allow(unused)]
    pub fn new(log_fn: unsafe extern "C" fn(u32, *const u8, usize)) -> Self {
        Self { log_fn }
    }

    /// Initializes the logger with the `log` framework.
    ///
    /// # Arguments
    ///
    /// * `max_level`: An optional maximum logging level.
    ///
    /// # Examples
    ///
    /// ```
    /// use ves_proto_common::log::LogLevel;
    /// use ves_proto_logger::Logger;
    /// use log::info;
    ///
    /// #[link(wasm_import_module = "log")]
    /// extern "C" {
    ///     #[link_name = "log"]
    ///     fn log_fn(level: u32, ptr: *const u8, len: usize);
    /// }
    ///
    /// fn start_game() {
    ///     Logger::new(log_fn).init(Some(LogLevel::Info)).unwrap();
    ///     info!("Logging initialized!");
    /// }
    /// ```
    #[allow(unused)]
    pub fn init(
        mut self,
        max_level: Option<ves_proto_common::log::LogLevel>,
    ) -> Result<(), SetLoggerError> {
        log::set_max_level(Self::map_filter_level(max_level));
        log::set_boxed_logger(Box::new(self))
    }

    fn map_filter_level(max_level: Option<ves_proto_common::log::LogLevel>) -> log::LevelFilter {
        if let Some(level) = max_level {
            use ves_proto_common::log::LogLevel;
            match level {
                LogLevel::Error => LevelFilter::Error,
                LogLevel::Warn => LevelFilter::Warn,
                LogLevel::Info => LevelFilter::Info,
                LogLevel::Debug => LevelFilter::Debug,
                LogLevel::Trace => LevelFilter::Trace,
            }
        } else {
            log::LevelFilter::Off
        }
    }

    fn send(&self, level: ves_proto_common::log::LogLevel, message: &str) {
        unsafe {
            (self.log_fn)(level.into(), message.as_ptr(), message.len());
        }
    }
}

impl Log for Logger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let level = record.metadata().level();
        match level {
            Level::Error => {}
            Level::Warn => {}
            Level::Info => {}
            Level::Debug => {}
            Level::Trace => {}
        }

        let message = format!("{}", record.args());

        self.send(record.metadata().level().into(), &message);
    }

    fn flush(&self) {
        // Do nothing
    }
}
