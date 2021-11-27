use std::ffi;
use std::io;
use std::str;

use crate::{buffer_with, Facility, LevelToSeverity, LogOption};
use flexi_logger::{writers::LogWriter, DeferredNow, FormatFunction, Record};

/// A Writer that uses [libc::syslog] to write syslog messages
pub struct Writer {
    /// Fn that maps [log::Level] to [crate::Severity]
    level_to_severity: LevelToSeverity,
    /// The maximum log level to allow through to syslog
    max_log_level: log::LevelFilter,
    /// fn to format a single [Record]
    format_function: FormatFunction,
}

impl Writer {
    pub fn try_new(
        ident: &str,
        facility: Facility,
        options: LogOption,
        level_to_severity: LevelToSeverity,
        max_log_level: log::LevelFilter,
        format_function: FormatFunction,
    ) -> Result<Self, ffi::NulError> {
        let ident = ffi::CString::new(ident.as_bytes())?;

        unsafe {
            libc::openlog(ident.into_raw(), options.bits(), facility.into());
        }

        Ok(Self {
            level_to_severity,
            max_log_level,
            format_function,
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

impl LogWriter for Writer {
    fn write(&self, now: &mut DeferredNow, record: &Record) -> io::Result<()> {
        let severity = (self.level_to_severity)(record.level());

        buffer_with(|tl_buf| match tl_buf.try_borrow_mut() {
            Ok(mut buffer) => {
                (self.format_function)(&mut *buffer, now, record).unwrap();
                let s = str::from_utf8(&buffer)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
                    .unwrap();

                unsafe {
                    libc::syslog(severity.into(), s.as_ptr() as *const libc::c_char);
                }
                buffer.clear();
            }
            Err(_) => {
                panic!();
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
