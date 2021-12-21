use std::ffi;
use std::io;
use std::str;

use crate::{buffer_with, Facility, LevelToSeverity, LogOption};
use flexi_logger::{writers::LogWriter, DeferredNow, FormatFunction, Record};

/// A Writer that uses [libc::syslog] to write syslog messages.
pub struct Writer {
    /// Fn that maps [log::Level] to [crate::Severity].
    level_to_severity: LevelToSeverity,
    /// The maximum log level to allow through to syslog.
    max_log_level: log::LevelFilter,
    /// fn to format a single [Record].
    format_function: FormatFunction,
    /// if defined the str given to the Writer will be truncated to this amount of bytes before submitting.
    max_bytes: Option<usize>,
}

/// Builds a Writer.
/// `ident `defaults to an empty string.
/// `facility `defaults to Facility::UserLevel this is the default for libc::openlog is passed facility 0.
/// `options` defaults to
pub struct Builder {
    /// String to identify the source of log messages submitted through the generated Writer.
    /// Typically the name of the executable.
    ident: Option<String>,
    facility: Facility,
    options: LogOption,
    level_to_severity: LevelToSeverity,
    max_log_level: log::LevelFilter,
    format_function: FormatFunction,
    max_bytes: Option<usize>,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            ident: None,
            facility: Facility::default(),
            options: LogOption::default(),
            level_to_severity: crate::default_level_mapping,
            max_log_level: log::LevelFilter::Info,
            format_function: crate::default_format,
            max_bytes: None,
        }
    }
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ident(mut self, ident: String) -> Self {
        self.ident = Some(ident);
        self
    }

    pub fn facility(mut self, facility: Facility) -> Self {
        self.facility = facility;
        self
    }

    pub fn options(mut self, options: LogOption) -> Self {
        self.options = options;
        self
    }

    pub fn level_to_severity(mut self, level_to_severity: LevelToSeverity) -> Self {
        self.level_to_severity = level_to_severity;
        self
    }

    pub fn max_log_level(mut self, max_log_level: log::LevelFilter) -> Self {
        self.max_log_level = max_log_level;
        self
    }

    pub fn format_function(mut self, format_function: FormatFunction) -> Self {
        self.format_function = format_function;
        self
    }

    pub fn max_bytes(mut self, max_bytes: impl Into<Option<usize>>) -> Self {
        self.max_bytes = max_bytes.into();
        self
    }

    /// Consume Vuiler into a Writer
    pub fn build(self) -> Result<Writer, ffi::NulError> {
        Writer::try_new(
            self.ident,
            self.facility,
            self.options,
            self.level_to_severity,
            self.max_log_level,
            self.format_function,
            self.max_bytes,
        )
    }
}

impl Writer {
    /// Returns a Writer or an error if the conversion of the ident &str fails due to the bytes containing a zero byte.
    pub fn try_new<'a>(
        ident: impl Into<Option<String>>,
        facility: Facility,
        options: LogOption,
        level_to_severity: LevelToSeverity,
        max_log_level: log::LevelFilter,
        format_function: FormatFunction,
        max_bytes: impl Into<Option<usize>>,
    ) -> Result<Self, ffi::NulError> {
        let ident = ident
            .into()
            .map(|s| ffi::CString::new(s.as_bytes()))
            .transpose()?;

        unsafe {
            libc::openlog(
                ident.map_or(std::ptr::null(), |s| s.into_raw()),
                options.bits,
                facility.into(),
            );
        }

        Ok(Self {
            level_to_severity,
            max_log_level,
            format_function,
            max_bytes: max_bytes.into(),
        })
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        unsafe {
            libc::closelog();
        }
    }
}

/// Find the first char boundary from max index
fn find_char_boundary_back_from_index<'a>(s: &'a str, mut max: usize) -> usize {
    if max >= s.len() {
        s.len()
    } else {
        while !s.is_char_boundary(max) {
            max -= 1;
        }
        max
    }
}

fn buffer_to_cstr<'a>(
    buffer: &'a mut Vec<u8>,
    max_bytes: Option<usize>,
) -> Result<&'a ffi::CStr, std::str::Utf8Error> {
    let new_buf_len = if let Some(max_bytes) = max_bytes {
        let char_index = find_char_boundary_back_from_index(str::from_utf8(&buffer)?, max_bytes);
        char_index + 1
    } else {
        buffer.len() + 1
    };

    buffer.resize(new_buf_len, 0);
    buffer[new_buf_len - 1] = 0;

    // Safety: the buffer will always have a zero byte
    Ok(unsafe { ffi::CStr::from_bytes_with_nul_unchecked(buffer) })
}

impl LogWriter for Writer {
    fn write(&self, now: &mut DeferredNow, record: &Record) -> io::Result<()> {
        let severity = (self.level_to_severity)(record.level());

        buffer_with(|tl_buf| match tl_buf.try_borrow_mut() {
            Ok(mut buffer) => {
                (self.format_function)(&mut *buffer, now, record).unwrap();
                let cstr = buffer_to_cstr(&mut buffer, self.max_bytes)
                    .expect("Failed to convert buffer to valid UTF8 string");

                unsafe {
                    libc::syslog(severity.into(), cstr.as_ptr());
                }
                buffer.clear();
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
