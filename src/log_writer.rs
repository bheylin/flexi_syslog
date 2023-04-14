//! The LogWriter that adapts flexi-logger log records to the syslog.
use std::{
    fmt,
    io::{self, ErrorKind},
    sync::Arc,
};

use arrayvec::ArrayVec;
use flexi_logger::{DeferredNow, Record};
use parking_lot::Mutex;
use syslog_fmt::v5424;
use syslog_net::{ReconnectionStrategy, Transport};

use crate::LevelToSeverity;

/// What should happen when the buffer runs out of space?
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum BufferWriteErrorStrategy {
    /// Ignore the Error. Adopting this strategy will result in as much of the syslog message
    /// being written as possible.
    Ignore,
    /// Pass the Error upwards
    Fail,
}

/// Writes [records](flexi_logger::Record) to the syslog through one of the available [transports](syslog_net::Transport).
///
/// Each record is formatted into a user message using the format_fn.
/// The user message is then [foratted](syslog::Formatter5424) into an [rfc3164](https://datatracker.ietf.org/doc/html/rfc5424) string
/// and sent to syslog through the transport.
pub struct LogWriter<const CAP: usize, RS> {
    /// Formats the syslog entry including metadata and user message
    formatter: v5424::Formatter,
    /// A transport and associated buffer for sending syslog messages.
    /// This is synced betwen threads using a Mutex.
    buffered_transport: Arc<Mutex<BufferedTransport<RS, CAP>>>,
    /// The maximum log level to allow through to syslog.
    max_log_level: log::LevelFilter,
    /// Fn that maps [log::Level] to [crate::Severity].
    level_to_severity: LevelToSeverity,
    /// How should a full buffer error be handled?
    /// Ignoring the error will truncate the message to the len of the buffer.
    buffer_write_error_strategy: BufferWriteErrorStrategy,
}

impl<const CAP: usize, RS: ReconnectionStrategy> LogWriter<CAP, RS> {
    /// Create a new LogWriter
    pub fn new(
        formatter: v5424::Formatter,
        transport: Transport,
        reconnection_strategy: RS,
        max_log_level: log::LevelFilter,
        level_to_severity: LevelToSeverity,
        buffer_write_error_strategy: BufferWriteErrorStrategy,
    ) -> LogWriter<CAP, RS> {
        Self {
            formatter,
            buffered_transport: Arc::new(Mutex::new(BufferedTransport::from_transport(
                transport,
                reconnection_strategy,
            ))),
            max_log_level,
            level_to_severity,
            buffer_write_error_strategy,
        }
    }
}

/// A synchronized set of resources needed to send a message in the syslog format and
/// to reconnect to broken transport sockets.
///
/// These resouces are, a byte buffer for constructing the syslog message and
/// a reconnection strategy object for applying different reconnection strategies.
struct BufferedTransport<RS, const CAP: usize> {
    buf: ArrayVec<u8, CAP>,
    transport: Option<Transport>,
    /// A strategy to create and reconnect to Transports
    reconnection_strategy: RS,
}

impl<RS: ReconnectionStrategy, const CAP: usize> BufferedTransport<RS, CAP> {
    fn from_transport(transport: Transport, reconnection_strategy: RS) -> Self {
        Self {
            buf: ArrayVec::<_, CAP>::new(),
            transport: Some(transport),
            reconnection_strategy,
        }
    }

    fn transport(&mut self) -> io::Result<&mut Transport> {
        self.transport.as_mut().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "Transport should only be None if reconnection has failed",
            )
        })
    }

    /// Try reconnected to the given transport if an error occurs while trying to send a message
    fn try_reconnect(&mut self, err: io::Error) -> io::Result<()> {
        let transport = self.transport.take();
        let new_transport = self.reconnection_strategy.reconnect(transport, err)?;
        self.transport.replace(new_transport);

        Ok(())
    }
}

impl<const CAP: usize, RS> fmt::Debug for LogWriter<CAP, RS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LogWriter")
            .field("formatter", &self.formatter)
            .field("max_log_level", &self.max_log_level)
            .finish()
    }
}

impl<const CAP: usize, RS: ReconnectionStrategy> flexi_logger::writers::LogWriter
    for LogWriter<CAP, RS>
{
    fn write(&self, _now: &mut DeferredNow, record: &Record<'_>) -> io::Result<()> {
        let mut buf_trans = self.buffered_transport.lock();
        let bt = &mut *buf_trans;
        let severity = (self.level_to_severity)(record.level());

        bt.buf.clear();

        if let Err(e) = self
            .formatter
            .format(&mut bt.buf, severity, record.args(), None)
        {
            if e.kind() == ErrorKind::WriteZero {
                match self.buffer_write_error_strategy {
                    BufferWriteErrorStrategy::Ignore => (),
                    BufferWriteErrorStrategy::Fail => return Err(e),
                }
            } else {
                return Err(e);
            }
        }

        if bt.transport.is_none() {
            if let Err(e) = bt.try_reconnect(io::ErrorKind::Other.into()) {
                eprintln!("Error while trying to reconnect to socket: {e}");
            }
        }

        let transport = bt.transport.as_mut().unwrap();

        match transport.send(&bt.buf) {
            Ok(bytes_written) => {
                eprintln!("Wrote bytes {bytes_written}");
            }
            Err(e) => {
                eprintln!("Transport Error occured while sending message; err: {e}",);
                if let Err(e) = bt.try_reconnect(e) {
                    eprintln!("Error while trying to reconnect to socket: {e}");
                }
            }
        }

        Ok(())
    }

    fn flush(&self) -> io::Result<()> {
        let mut buf_trans = self.buffered_transport.lock();
        buf_trans.transport()?.flush()
    }

    fn max_log_level(&self) -> log::LevelFilter {
        self.max_log_level
    }
}
