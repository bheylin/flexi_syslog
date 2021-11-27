mod libc_writer;
mod log_option;

use core::cell::RefCell;
use flexi_logger::{DeferredNow, Record};
use std::io;

pub use libc_writer::Writer as LibcWriter;
pub use log_option::LogOption;

/// Signature for a custom mapping function that maps the rust log levels to
/// values of the syslog Severity.
pub type LevelToSeverity = fn(level: log::Level) -> Severity;

/// Syslog Facility.
///
/// See [RFC 5424](https://datatracker.ietf.org/doc/rfc5424).
#[derive(Copy, Clone, Debug)]
pub enum Facility {
    /// kernel messages.
    Kernel,
    /// user-level messages.
    UserLevel,
    /// mail system.
    MailSystem,
    /// system daemons.
    SystemDaemons,
    /// security/authorization messages.
    Auth,
    /// messages generated internally by syslogd.
    SyslogD,
    /// line printer subsystem.
    LinePrinter,
    /// network news subsystem.
    News,
    /// UUCP subsystem.
    Uucp,
    /// security/authorization messages.
    AuthPriv,
    /// FTP daemon.
    Ftp,
    /// log alert.
    Alert,
    /// local use 0  (local0).
    Local0,
    /// local use 1  (local1).
    Local1,
    /// local use 2  (local2).
    Local2,
    /// local use 3  (local3).
    Local3,
    /// local use 4  (local4).
    Local4,
    /// local use 5  (local5).
    Local5,
    /// local use 6  (local6).
    Local6,
    /// local use 7  (local7).
    Local7,
}

impl From<Facility> for libc::c_int {
    fn from(f: Facility) -> Self {
        match f {
            Facility::Kernel => libc::LOG_KERN,
            Facility::UserLevel => libc::LOG_USER,
            Facility::MailSystem => libc::LOG_MAIL,
            Facility::SystemDaemons => libc::LOG_DAEMON,
            Facility::Auth => libc::LOG_AUTH,
            Facility::SyslogD => libc::LOG_SYSLOG,
            Facility::LinePrinter => libc::LOG_LPR,
            Facility::News => libc::LOG_NEWS,
            Facility::Uucp => libc::LOG_UUCP,
            Facility::AuthPriv => libc::LOG_AUTHPRIV,
            Facility::Ftp => libc::LOG_FTP,
            Facility::Alert => libc::LOG_ALERT,
            Facility::Local0 => libc::LOG_LOCAL0,
            Facility::Local1 => libc::LOG_LOCAL1,
            Facility::Local2 => libc::LOG_LOCAL2,
            Facility::Local3 => libc::LOG_LOCAL3,
            Facility::Local4 => libc::LOG_LOCAL4,
            Facility::Local5 => libc::LOG_LOCAL5,
            Facility::Local6 => libc::LOG_LOCAL6,
            Facility::Local7 => libc::LOG_LOCAL7,
        }
    }
}

/// [`SyslogConnector`]'s severity.
///
/// See [RFC 5424](https://datatracker.ietf.org/doc/rfc5424).
#[derive(Debug)]
pub enum Severity {
    /// System is unusable.
    Emergency,
    /// Action must be taken immediately.
    Alert,
    /// Critical conditions.
    Critical,
    /// Error conditions.
    Error,
    /// Warning conditions
    Warning,
    /// Normal but significant condition
    Notice,
    /// Informational messages.
    Info,
    /// Debug-level messages.
    Debug,
}

impl From<Severity> for libc::c_int {
    fn from(s: Severity) -> Self {
        match s {
            Severity::Emergency => libc::LOG_EMERG,
            Severity::Alert => libc::LOG_ALERT,
            Severity::Critical => libc::LOG_CRIT,
            Severity::Error => libc::LOG_ERR,
            Severity::Warning => libc::LOG_WARNING,
            Severity::Notice => libc::LOG_NOTICE,
            Severity::Info => libc::LOG_INFO,
            Severity::Debug => libc::LOG_DEBUG,
        }
    }
}

/// A default formatter if you don't want to think to hard about it
/// {record.level} {record.target} l:{record.line} {record.args}
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
            .map(ToString::to_string)
            .unwrap_or_else(|| "-".to_owned()),
        record.args()
    )
}

/// A default mapping from [log::Level] to [Severity]
pub fn default_level_mapping(level: log::Level) -> Severity {
    match level {
        log::Level::Error => Severity::Error,
        log::Level::Warn => Severity::Warning,
        log::Level::Info => Severity::Info,
        log::Level::Debug | log::Level::Trace => Severity::Debug,
    }
}

///
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
