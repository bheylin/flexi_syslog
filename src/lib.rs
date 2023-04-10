//! A flexi-logger LogWriter that formats and transports log records to the syslog using the syslog crate.
pub mod log_writer;

use core::cell::RefCell;
use std::io;

use flexi_logger::{DeferredNow, Record};
use syslog_fmt::Severity;

pub use log_writer::LogWriter;

/// Signature for a custom mapping function that maps the rust log levels to
/// values of the syslog Severity.
pub type LevelToSeverity = fn(level: log::Level) -> Severity;

/// A default formatter if you don't want to think to hard about it.
/// Format: {record.level} {record.target} l:{record.line} {record.args}
pub fn default_format(
    w: &mut dyn io::Write,
    _now: &mut DeferredNow,
    record: &Record<'_>,
) -> Result<(), io::Error> {
    write!(
        w,
        "{} {} l:{} {}",
        record.level(),
        record.target(),
        record
            .line()
            .as_ref()
            .map_or_else(|| "-".to_owned(), ToString::to_string),
        record.args()
    )
}

/// A default mapping from [log::Level] to [Severity]
pub fn default_level_mapping(level: log::Level) -> Severity {
    match level {
        log::Level::Error => Severity::Err,
        log::Level::Warn => Severity::Warning,
        log::Level::Info => Severity::Info,
        log::Level::Debug | log::Level::Trace => Severity::Debug,
    }
}

// Thread-local buffer
pub(crate) fn buffer_with<F>(f: F)
where
    F: FnOnce(&RefCell<Vec<u8>>),
{
    const DEFAULT_BUFFER_BYTES: usize = 2 * 1024;

    thread_local! {
        static BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(DEFAULT_BUFFER_BYTES));
    }
    BUFFER.with(f);
}
