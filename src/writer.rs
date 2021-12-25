use std::fmt;
use std::io;
use std::str;
use std::sync::{Arc, Mutex};

use flexi_logger::{writers::LogWriter, DeferredNow, Record};
use syslog::LogFormat;

use crate::{buffer_with, LevelToSeverity};

/// A Writer that wraps [syslog::Logger] and writes to the backend using the syslog Formatter.
pub struct Writer<Backend, Formatter>
where
    for<'a> Formatter: LogFormat<StrWriter<'a>>,
    Backend: io::Write + Send + Sync,
{
    /// Fn that maps [log::Level] to [crate::Severity].
    level_to_severity: LevelToSeverity,
    /// The backend to write log messages to and a formatter.
    logger: Arc<Mutex<syslog::Logger<Backend, Formatter>>>,
    // /// if defined the str given to the Writer will be truncated to this amount of bytes before submitting.
    // max_bytes: Option<usize>,
    /// The maximum log level to allow through to syslog.
    max_log_level: log::LevelFilter,
}

/// Builds a Writer.
pub struct Builder {
    /// String to identify the source of log messages submitted through the generated Writer.
    /// Typically the name of the executable.
    level_to_severity: LevelToSeverity,
    max_bytes: Option<usize>,
    max_log_level: log::LevelFilter,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            level_to_severity: crate::default_level_mapping,
            max_bytes: None,
            max_log_level: log::LevelFilter::Info,
        }
    }
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
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

    /// Consume Builer into a Writer
    pub fn build<Backend, Formatter>(
        self,
        logger: syslog::Logger<Backend, Formatter>,
    ) -> Writer<Backend, Formatter>
    where
        for<'a> Formatter: LogFormat<StrWriter<'a>>,
        Backend: io::Write + Send + Sync,
    {
        Writer::new(
            self.level_to_severity,
            self.max_bytes,
            self.max_log_level,
            logger,
        )
    }
}

impl<Backend, Formatter> Writer<Backend, Formatter>
where
    for<'a> Formatter: LogFormat<StrWriter<'a>>,
    Backend: io::Write + Send + Sync,
{
    /// Returns a Writer.
    pub fn new(
        level_to_severity: LevelToSeverity,
        _max_bytes: impl Into<Option<usize>>,
        max_log_level: log::LevelFilter,
        logger: syslog::Logger<Backend, Formatter>,
    ) -> Self {
        Self {
            level_to_severity,
            // max_bytes: max_bytes.into(),
            max_log_level,
            logger: Arc::new(Mutex::new(logger)),
        }
    }
}

// /// Find the first char boundary from max index
// fn find_char_boundary_back_from_index<'a>(s: &'a str, mut max: usize) -> usize {
//     if max >= s.len() {
//         s.len()
//     } else {
//         while !s.is_char_boundary(max) {
//             max -= 1;
//         }
//         max
//     }
// }

// fn buffer_to_cstr<'a>(
//     buffer: &'a mut Vec<u8>,
//     max_bytes: Option<usize>,
// ) -> Result<&'a ffi::CStr, std::str::Utf8Error> {
//     let new_buf_len = if let Some(max_bytes) = max_bytes {
//         let char_index = find_char_boundary_back_from_index(str::from_utf8(&buffer)?, max_bytes);
//         char_index + 1
//     } else {
//         buffer.len() + 1
//     };

//     buffer.resize(new_buf_len, 0);
//     buffer[new_buf_len - 1] = 0;

//     // Safety: the buffer will always have a zero byte
//     Ok(unsafe { ffi::CStr::from_bytes_with_nul_unchecked(buffer) })
// }

#[derive(Debug)]
pub struct StrWriter<'a> {
    s: &'a str,
}

impl<'a> fmt::Display for StrWriter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.write_str(self.s)
    }
}

impl<Backend, Formatter> LogWriter for Writer<Backend, Formatter>
where
    for<'a> Formatter: LogFormat<StrWriter<'a>> + Send + Sync,
    Backend: io::Write + Send + Sync,
{
    fn write(&self, _now: &mut DeferredNow, record: &Record) -> io::Result<()> {
        let severity = (self.level_to_severity)(record.level());

        buffer_with(|tl_string| match tl_string.try_borrow_mut() {
            Ok(mut string) => {
                use core::fmt::Write;
                use syslog::Severity;

                string.clear();
                write!(*string, "{}", record.args())
                    .expect("Failed to write record args to string buffer");
                // let cstr = buffer_to_cstr(&mut buffer, self.max_bytes)
                //     .expect("Failed to convert buffer to valid UTF8 string");

                let str_writer = StrWriter { s: &*string };
                let mut logger = self.logger.lock().expect("Failed to lock logger Mutex");

                let res = match severity {
                    Severity::LOG_EMERG => logger.emerg(str_writer),
                    Severity::LOG_ALERT => logger.alert(str_writer),
                    Severity::LOG_CRIT => logger.crit(str_writer),
                    Severity::LOG_ERR => logger.err(str_writer),
                    Severity::LOG_WARNING => logger.warning(str_writer),
                    Severity::LOG_NOTICE => logger.notice(str_writer),
                    Severity::LOG_INFO => logger.info(str_writer),
                    Severity::LOG_DEBUG => logger.debug(str_writer),
                };

                res.expect("Failed to write message to syslog::Logger");

                // LogFormat::format(
                //     &mut logger.formatter,
                //     &mut logger.backend,
                //     severity,
                //     str_writer,
                // );

                // logger
                //     .formatter
                //     .format(&mut logger.backend, severity, str_writer);

                string.clear();
            }
            Err(e) => {
                panic!("{}", e.to_string());
            }
        });

        Ok(())
    }

    fn flush(&self) -> io::Result<()> {
        Ok(())
    }

    fn max_log_level(&self) -> log::LevelFilter {
        self.max_log_level
    }
}
