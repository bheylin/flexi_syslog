//! A flexi-logger LogWriter that formats and transports log records to the syslog using the syslog crate.
#![allow(rustdoc::private_intra_doc_links)]
#![deny(future_incompatible)]
#![deny(missing_debug_implementations)]
#![deny(nonstandard_style)]
#![deny(rust_2021_compatibility)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(unsafe_code)]
#![deny(unused)]
#![deny(warnings)]

pub mod log_writer;

use core::cell::RefCell;
use std::io;

use flexi_logger::{DeferredNow, Record};
use syslog::Severity;

pub use log_writer::LogWriter;

/// Signature for a custom mapping function that maps the rust log levels to
/// values of the syslog Severity.
pub type LevelToSeverity = fn(level: log::Level) -> Severity;

/// A default formatter if you don't want to think to hard about it.
/// Format: {record.level} {record.target} l:{record.line} {record.args}
pub fn default_format(
    w: &mut dyn io::Write,
    _now: &mut DeferredNow,
    record: &Record,
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
        log::Level::Error => Severity::LOG_ERR,
        log::Level::Warn => Severity::LOG_WARNING,
        log::Level::Info => Severity::LOG_INFO,
        log::Level::Debug | log::Level::Trace => Severity::LOG_DEBUG,
    }
}

/// Return the executable name.
pub fn exe_name_from_env() -> io::Result<String> {
    std::env::current_exe()?
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "exe path has no filename"))?
        .to_str()
        .map(String::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "exe name is not valid UTF8"))
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
