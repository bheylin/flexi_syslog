use flexi_syslog::exe_name_from_env;

use flexi_logger::Logger;

fn main() -> anyhow::Result<()> {
    use flexi_syslog as syslog;

    let ident = exe_name_from_env()?;
    let syslog_writer = syslog::LibcWriter::try_new(
        &ident,
        syslog::Facility::Local0,
        syslog::LogOption::LOG_CONS | syslog::LogOption::LOG_PID,
        syslog::default_level_mapping,
        log::LevelFilter::Info,
        syslog::default_format,
    )?;

    let logger = Logger::try_with_str("info")?.log_to_writer(Box::new(syslog_writer));
    logger.start()?;

    log::info!("Info gets through");
    log::trace!("Trace is filtered");

    Ok(())
}
