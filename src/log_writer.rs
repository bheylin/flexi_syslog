//! The LogWriter that adapts flexi-logger log records to the syslog.
use std::{fmt, io, sync::Arc};

use arrayvec::ArrayVec;
use flexi_logger::{DeferredNow, Record};
use parking_lot::Mutex;
use syslog_fmt::v5424;
use syslog_net::Transport;

use crate::LevelToSeverity;

/// Writes [records](flexi_logger::Record) to the syslog through one of the available [transports](syslog_net::Transport).
///
/// Each record is formatted into a user message using the format_fn.
/// The user message is then [foratted](syslog::Formatter5424) into an [rfc3164](https://datatracker.ietf.org/doc/html/rfc5424) string
/// and sent to syslog through the transport.
pub struct LogWriter<const CAP: usize> {
    /// Formats the syslog entry including metadata and user message
    formatter: v5424::Formatter,
    /// transport for sending syslog messages
    buffered_transport: Arc<Mutex<BufferedTransport<CAP>>>,
    /// The maximum log level to allow through to syslog.
    max_log_level: log::LevelFilter,
    /// Fn that maps [log::Level] to [crate::Severity].
    level_to_severity: LevelToSeverity,
}

struct BufferedTransport<const CAP: usize> {
    buf: ArrayVec<u8, CAP>,
    transport: Transport,
}

impl<const CAP: usize> LogWriter<CAP> {
    pub fn new(
        formatter: v5424::Formatter,
        transport: Transport,
        max_log_level: log::LevelFilter,
        level_to_severity: LevelToSeverity,
    ) -> LogWriter<CAP> {
        let buf = ArrayVec::<_, CAP>::new();
        Self {
            formatter,
            buffered_transport: Arc::new(Mutex::new(BufferedTransport { buf, transport })),
            max_log_level,
            level_to_severity,
        }
    }
}

impl<const CAP: usize> fmt::Debug for LogWriter<CAP> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LogWriter")
            .field("formatter", &self.formatter)
            .field("max_log_level", &self.max_log_level)
            .finish()
    }
}

impl<const CAP: usize> flexi_logger::writers::LogWriter for LogWriter<CAP> {
    fn write(&self, _now: &mut DeferredNow, record: &Record<'_>) -> io::Result<()> {
        let mut buf_trans = self.buffered_transport.lock();
        let bt = &mut *buf_trans;
        let severity = (self.level_to_severity)(record.level());

        bt.buf.clear();

        self.formatter
            .format(&mut bt.buf, severity, record.args(), None)?;

        bt.transport.send(&bt.buf)?;

        Ok(())
    }

    fn flush(&self) -> io::Result<()> {
        let mut buf_trans = self.buffered_transport.lock();

        buf_trans.transport.flush()
    }

    fn max_log_level(&self) -> log::LevelFilter {
        self.max_log_level
    }
}
