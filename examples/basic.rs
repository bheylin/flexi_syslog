use flexi_syslog::exe_name_from_env;

use flexi_logger::Logger;

fn main() -> anyhow::Result<()> {
    use flexi_syslog as syslog;

    let syslog_writer = syslog::Builder::new()
        .ident(exe_name_from_env()?)
        .facility(syslog::Facility::Local0)
        .options(syslog::LogOption::LOG_CONS | syslog::LogOption::LOG_PID)
        .level_to_severity(syslog::default_level_mapping)
        .max_log_level(log::LevelFilter::Info)
        .format_function(syslog::default_format)
        .build()?;

    let logger = Logger::try_with_str("info")?.log_to_writer(Box::new(syslog_writer));
    logger.start()?;

    log::info!("Info gets through");
    log::trace!("Trace is filtered");

    Ok(())
}
