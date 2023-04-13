//! A flexi-logger LogWriter that formats and transports log records to the syslog using the syslog crate.
mod log_writer;

use syslog_fmt::Severity;

pub use log_writer::{BrokenPipeErrorStrategy, FullBufferErrorStrategy, LogWriter};

/// Signature for a custom mapping function that maps the rust log levels to
/// values of the syslog Severity.
pub type LevelToSeverity = fn(level: log::Level) -> Severity;

/// A default mapping from [log::Level] to [Severity]
pub fn default_level_mapping(level: log::Level) -> Severity {
    match level {
        log::Level::Error => Severity::Err,
        log::Level::Warn => Severity::Warning,
        log::Level::Info => Severity::Info,
        log::Level::Debug | log::Level::Trace => Severity::Debug,
    }
}
