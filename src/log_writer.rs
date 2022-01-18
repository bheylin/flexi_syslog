//! The LogWriter that adapts flexi-logger log records to the syslog.
use std::fmt;
use std::io;
use std::str;
use std::sync::{Arc, Mutex};

use flexi_logger::{DeferredNow, FormatFunction, Record};

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
    backend: Arc<Mutex<Backend>>,
    /// Fn to format a single [Record] into the message section of a syslog entry.
    format_fn: FormatFunction,
    /// Formats the syslog entry including metadata and user message
    formatter: syslog::Formatter5424,
    /// Fn that maps [log::Level] to [crate::Severity].
    level_to_severity: LevelToSeverity,
    /// if defined truncate the bytes sent to the bacnend to be at most this max.
    max_bytes: Option<usize>,
    /// The maximum log level to allow through to syslog.
    max_log_level: log::LevelFilter,
}

impl<Backend> fmt::Debug for LogWriter<Backend>
where
    Backend: io::Write + Send + Sync,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LogWriter")
            .field("formatter", &self.formatter)
            .field("max_bytes", &self.max_bytes)
            .field("max_log_level", &self.max_log_level)
            .finish()
    }
}

/// Builds a Writer.
pub struct Builder {
    /// Fn to format a single [Record] into the message section of a syslog entry.
    format_fn: FormatFunction,
    /// Fn that maps [log::Level] to [crate::Severity].
    level_to_severity: LevelToSeverity,
    /// if defined truncate the bytes sent to the bacnend to be at most this max.
    max_bytes: Option<usize>,
    /// The maximum log level to allow through to syslog.
    max_log_level: log::LevelFilter,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            format_fn: crate::default_format,
            level_to_severity: crate::default_level_mapping,
            max_bytes: None,
            max_log_level: log::LevelFilter::Info,
        }
    }
}

impl fmt::Debug for Builder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Builder")
            .field("max_bytes", &self.max_bytes)
            .field("max_log_level", &self.max_log_level)
            .finish()
    }
}

impl Builder {
    pub fn format(mut self, format_fn: FormatFunction) -> Self {
        self.format_fn = format_fn;
        self
    }

    pub fn level_to_severity(mut self, level_to_severity: LevelToSeverity) -> Self {
        self.level_to_severity = level_to_severity;
        self
    }

    pub fn max_bytes(mut self, max_bytes: impl Into<Option<usize>>) -> Self {
        self.max_bytes = max_bytes.into();
        self
    }

    pub fn max_log_level(mut self, max_log_level: log::LevelFilter) -> Self {
        self.max_log_level = max_log_level;
        self
    }

    /// Consume Builder into a Writer backed by the given syslog logger.
    pub fn build<Backend>(
        self,
        logger: syslog::Logger<Backend, syslog::Formatter5424>,
    ) -> LogWriter<Backend>
    where
        Backend: io::Write + Send + Sync,
    {
        LogWriter::new(
            self.format_fn,
            self.level_to_severity,
            self.max_bytes,
            self.max_log_level,
            logger.formatter,
            logger.backend,
        )
    }
}

impl<Backend> LogWriter<Backend>
where
    Backend: io::Write + Send + Sync,
{
    /// Returns a Writer.
    pub fn new(
        format_fn: FormatFunction,
        level_to_severity: LevelToSeverity,
        max_bytes: impl Into<Option<usize>>,
        max_log_level: log::LevelFilter,
        formatter: syslog::Formatter5424,
        backend: Backend,
    ) -> Self {
        Self {
            format_fn,
            level_to_severity,
            max_bytes: max_bytes.into(),
            max_log_level,
            formatter,
            backend: Arc::new(Mutex::new(backend)),
        }
    }
}

impl<Backend> flexi_logger::writers::LogWriter for LogWriter<Backend>
where
    Backend: io::Write + Send + Sync,
{
    fn write(&self, now: &mut DeferredNow, record: &Record) -> io::Result<()> {
        use syslog::LogFormat;

        let severity = (self.level_to_severity)(record.level());

        buffer_with(|tl_bytes| match tl_bytes.try_borrow_mut() {
            Ok(mut bytes) => {
                bytes.clear();

                if let Some(max_bytes) = self.max_bytes {
                    let mut byte_writer = MaxByteWriter::new(&mut *bytes, max_bytes);
                    (self.format_fn)(&mut byte_writer, now, record)
                } else {
                    (self.format_fn)(&mut *bytes, now, record)
                }
                .expect("Failed to format flexi_logger::Record");

                let s = str::from_utf8(&*bytes)
                    .expect("Failed to convert message bytes into valid str");

                let mut backend = self
                    .backend
                    .lock()
                    .expect("Failed to lock syslog backend Mutex");

                let data = std::collections::HashMap::default();

                self.formatter
                    .format(&mut *backend, severity, (0, data, s))
                    .expect("Failed to format message");

                bytes.clear();
            }
            Err(e) => {
                panic!("{}", e.to_string());
            }
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

#[derive(Debug)]
/// Writes a maximum amount of bytes and will ignore the rest while claiming to have written them.
struct MaxByteWriter<W: io::Write> {
    bytes_remaining: usize,
    w: W,
}

impl<W: io::Write> MaxByteWriter<W> {
    pub fn new(w: W, max_bytes: usize) -> Self {
        Self {
            bytes_remaining: max_bytes,
            w,
        }
    }
}

impl<W: io::Write> io::Write for MaxByteWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.bytes_remaining == 0 {
            // if the maximum bytes written has been exceeded
            // pretend to write them
            Ok(buf.len())
        } else if buf.len() <= self.bytes_remaining {
            // the complete buffer can be written
            let bytes_written = self.w.write(buf)?;
            self.bytes_remaining -= bytes_written;
            Ok(bytes_written)
        } else {
            // there are bytes_remaining but it's less than the buffer.len()
            let i = find_char_boundary_from_end(&buf[..self.bytes_remaining]);
            let bytes_written = self.w.write(&buf[..=i])?;
            self.bytes_remaining -= bytes_written;
            Ok(buf.len())
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.w.flush()
    }
}

/// Find the first char boundary from the end of the buffer.
fn find_char_boundary_from_end(buf: &[u8]) -> usize {
    debug_assert_ne!(buf.len(), 0);
    if buf.len() == 1 {
        0
    } else {
        let mut i = buf.len() - 1;
        while i > 0 && !is_char_boundary(buf[i]) {
            i -= 1;
        }
        i
    }
}

fn is_char_boundary(b: u8) -> bool {
    b as i8 >= -0x40
}

#[cfg(test)]
mod max_byte_writer {
    use std::io::Write;

    use super::MaxByteWriter;

    #[test]
    fn should_truncate_on_overflow() {
        const MAX_BYTES: usize = 10;
        let input = "this is the end";
        let mut output: [u8; MAX_BYTES] = [101; MAX_BYTES];
        let mut w = MaxByteWriter::new(&mut output as &mut [u8], MAX_BYTES);
        let bytes_written = w.write(input.as_bytes()).unwrap();

        assert_eq!(bytes_written, 15);
        assert_eq!("this is th", std::str::from_utf8(&output).unwrap());
    }

    #[test]
    fn should_truncate_on_multi_write_overflow() {
        const MAX_BYTES: usize = 10;
        let input = "this is the end";
        let mut output: [u8; MAX_BYTES] = [101; MAX_BYTES];
        let mut w = MaxByteWriter::new(&mut output as &mut [u8], MAX_BYTES);

        let bytes = input.as_bytes();
        let chunk_a = &bytes[..=4];
        let chunk_b = &bytes[5..=11];
        let chunk_c = &bytes[12..];

        let bytes_written_a = w.write(chunk_a).unwrap();
        let bytes_written_b = w.write(chunk_b).unwrap();
        let bytes_written_c = w.write(chunk_c).unwrap();

        let bytes_written = bytes_written_a + bytes_written_b + bytes_written_c;

        assert_eq!(bytes_written, 15);
        assert_eq!("this is th", std::str::from_utf8(&output).unwrap());
    }

    #[test]
    fn should_write_all_input_on_underflow() {
        const MAX_BYTES: usize = 20;
        let input = "this is the end";
        let mut output: [u8; MAX_BYTES] = [101; MAX_BYTES];
        let mut w = MaxByteWriter::new(&mut output as &mut [u8], MAX_BYTES);
        let bytes_written = w.write(input.as_bytes()).unwrap();

        assert_eq!(bytes_written, 15);
        assert_eq!(output[bytes_written], 101);

        output[bytes_written] = 0;

        let s = std::ffi::CStr::from_bytes_with_nul(&output[..=bytes_written])
            .unwrap()
            .to_str()
            .unwrap();

        assert_eq!(s, input);
    }

    #[test]
    fn should_write_all_input_on_multi_write_underflow() {
        const MAX_BYTES: usize = 20;
        let input = "this is the end";
        let mut output: [u8; MAX_BYTES] = [101; MAX_BYTES];
        let mut w = MaxByteWriter::new(&mut output as &mut [u8], MAX_BYTES);

        let bytes = input.as_bytes();
        let chunk_a = &bytes[..=4];
        let chunk_b = &bytes[5..=11];
        let chunk_c = &bytes[12..];

        let bytes_written_a = w.write(chunk_a).unwrap();
        let bytes_written_b = w.write(chunk_b).unwrap();
        let bytes_written_c = w.write(chunk_c).unwrap();

        let bytes_written = bytes_written_a + bytes_written_b + bytes_written_c;

        assert_eq!(bytes_written, 15);
        assert_eq!(output[bytes_written], 101);

        output[bytes_written] = 0;

        let s = std::ffi::CStr::from_bytes_with_nul(&output[..=bytes_written])
            .unwrap()
            .to_str()
            .unwrap();

        assert_eq!(s, input);
    }

    #[test]
    fn should_write_nothing_on_empty_input() {
        const MAX_BYTES: usize = 10;
        let input = "";
        let mut output: [u8; MAX_BYTES] = [101; MAX_BYTES];
        let mut w = MaxByteWriter::new(&mut output as &mut [u8], MAX_BYTES);
        let bytes_written = w.write(input.as_bytes()).unwrap();
        assert_eq!(bytes_written, 0);
    }
}
