use std::cell::RefCell;
use std::io::Write;
use std::path::PathBuf;
use ves_proto_common::log::LogLevel;

struct LoggerInner {
    out: Box<dyn Write>,
    failed: bool,
}

pub struct Logger {
    // Wrapped in a refcell so that we don't have to ask for &mut self everywhere.
    inner: RefCell<LoggerInner>,
}

impl Logger {
    pub fn from_file(path: PathBuf) -> std::io::Result<Self> {
        Ok(Self {
            inner: RefCell::new(LoggerInner {
                out: Box::new(std::fs::File::create(path)?),
                failed: false,
            }),
        })
    }

    pub fn log(&self, level: LogLevel, msg: &str) {
        if let Err(err) = self.log_internal(level, msg) {
            // Don't write this more than once.
            if !self.inner.borrow().failed {
                eprintln!("Could not write to log: {}", err);
                self.inner.borrow_mut().failed = true;
            }
        }
    }

    fn log_internal(&self, level: LogLevel, msg: &str) -> std::io::Result<()> {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
        writeln!(&mut self.inner.borrow_mut().out, "{timestamp} [{level}] {msg}")
    }
}
