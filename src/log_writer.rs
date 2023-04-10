//! The LogWriter that adapts flexi-logger log records to the syslog.
use std::{
    fmt, io, str,
    sync::{Arc, Mutex},
};

use flexi_logger::{DeferredNow, FormatFunction, Record};
use syslog_fmt::v5424;

use crate::{buffer_with, LevelToSeverity};

/// Writes [records](flexi_logger::Record) to the given syslog [backend](syslog::LoggerBackend).
///
/// Each record is formatted into a user message using the format_fn.
/// The user message is then [foratted](syslog::Formatter3164) into an [rfc3164](https://datatracker.ietf.org/doc/html/rfc3164) string
/// and sent to syslog through the backend writer.
pub struct LogWriter<Backend>
where
    Backend: io::Write + Send + Sync,
{
    /// backend for sending syslog messages
    pub backend: Arc<Mutex<Backend>>,
    /// Fn to format a single [Record] into the message section of a syslog entry.
    pub format_fn: FormatFunction,
    /// Formats the syslog entry including metadata and user message
    pub formatter: v5424::Formatter,
    /// Fn that maps [log::Level] to [crate::Severity].
    pub level_to_severity: LevelToSeverity,
    /// The maximum log level to allow through to syslog.
    pub max_log_level: log::LevelFilter,
}

impl<Backend> fmt::Debug for LogWriter<Backend>
where
    Backend: io::Write + Send + Sync,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LogWriter")
            .field("formatter", &self.formatter)
            .field("max_log_level", &self.max_log_level)
            .finish()
    }
}

impl<Backend> flexi_logger::writers::LogWriter for LogWriter<Backend>
where
    Backend: io::Write + Send + Sync,
{
    fn write(&self, now: &mut DeferredNow, record: &Record<'_>) -> io::Result<()> {
        let severity = (self.level_to_severity)(record.level());

        buffer_with(|tl_bytes| {
            let mut bytes = match tl_bytes.try_borrow_mut() {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("{e}");
                    return;
                }
            };

            bytes.clear();

            let result = (self.format_fn)(&mut *bytes, now, record);

            if let Err(e) = result {
                eprintln!("Failed to format flexi_logger::Record; error: {e}");
                return;
            }

            let s = match str::from_utf8(&bytes) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to convert message bytes into valid str; error: {e}");
                    return;
                }
            };

            let mut backend = match self.backend.lock() {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Failed to lock backend Mutex while trying to log message; message: {s}, error: {e}");
                    return;
                }
            };

            if let Err(e) = self.formatter.format(&mut *backend, severity, s) {
                eprintln!("Failed to write message to syslog backend; {e}");
                return;
            }

            bytes.clear();
        });

        Ok(())
    }

    fn flush(&self) -> io::Result<()> {
        let mut backend = self
            .backend
            .lock()
            .expect("Failed to lock syslog backend Mutex");

        backend.flush()
    }

    fn max_log_level(&self) -> log::LevelFilter {
        self.max_log_level
    }
}
