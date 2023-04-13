//! A flexi-logger LogWriter that formats and transports log records to the syslog using the syslog crate.
mod log_writer;

use syslog_fmt::Severity;

pub use log_writer::{FullBufferErrorStrategy, LogWriter};

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

#[cfg(test)]
mod test {
    use std::os::unix::net::UnixDatagram;

    use crate::FullBufferErrorStrategy;

    #[test]
    fn should_log() {
        let (tx, rx) = UnixDatagram::pair().unwrap();

        let formatter = syslog_fmt::v5424::Formatter::new(
            syslog_fmt::Facility::User,
            "app.domain.com",
            "app_test",
            None,
        );

        let syslog_writer = crate::LogWriter::<1024>::new(
            formatter,
            tx.into(),
            log::LevelFilter::Info,
            crate::default_level_mapping,
            FullBufferErrorStrategy::Ignore,
        );

        let logger = flexi_logger::Logger::try_with_str("info")
            .expect("Failed to init logger")
            .log_to_writer(Box::new(syslog_writer));

        let handle = logger.start().unwrap();

        log::info!("Info gets through");
        log::trace!("Trace is filtered");

        handle.flush();

        let mut buf = vec![0u8; 128];
        let bytes_received = rx.recv(&mut buf).unwrap();
        buf.truncate(bytes_received);
        let s = String::from_utf8(buf).unwrap();
        assert!(s.ends_with("Info gets through"));
        assert!(bytes_received > 0);
    }
}
