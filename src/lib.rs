//! An implementation of a flexi_logger LogWriter that writes to syslog through the libc crate.
#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2021_compatibility)]
#![deny(unused)]
#![deny(warnings)]

mod writer;

use core::cell::RefCell;
use std::io;

use syslog::Severity;

pub use writer::{Builder, Writer};

/// Signature for a custom mapping function that maps the rust log levels to
/// values of the syslog Severity.
pub type LevelToSeverity = fn(level: log::Level) -> Severity;

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
    F: FnOnce(&RefCell<String>),
{
    const DEFAULT_BUFFER_BYTES: usize = 2 * 1024;

    thread_local! {
        static BUFFER: RefCell<String> = RefCell::new(String::with_capacity(DEFAULT_BUFFER_BYTES));
    }
    BUFFER.with(f);
}
