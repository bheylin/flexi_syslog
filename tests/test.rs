use std::io;

use flexi_syslog as syslog;
use flexi_syslog::exe_name_from_env;

pub fn default_builder() -> io::Result<syslog::Builder> {
    Ok(syslog::Builder::new()
        .ident(exe_name_from_env()?)
        .facility(syslog::Facility::Local0)
        .options(syslog::LogOption::LOG_CONS | syslog::LogOption::LOG_PID)
        .level_to_severity(syslog::default_level_mapping)
        .max_log_level(log::LevelFilter::Info)
        .format_function(syslog::default_format))
}
