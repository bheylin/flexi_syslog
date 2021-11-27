use flexi_logger::Logger;

#[test]
fn should_write() -> anyhow::Result<()> {
    let writer = test::new_writer()?;
    let logger = Logger::try_with_str("info")?.log_to_writer(Box::new(writer));
    logger.start()?;

    log::info!("Info gets through");
    log::trace!("Trace is filtered");

    Ok(())
}

mod test {
    use std::io;

    use flexi_syslog as syslog;
    use flexi_syslog::exe_name_from_env;

    pub fn new_writer() -> io::Result<syslog::LibcWriter> {
        let ident = exe_name_from_env()?;
        Ok(syslog::LibcWriter::try_new(
            &ident,
            syslog::Facility::Local0,
            syslog::LogOption::LOG_CONS | syslog::LogOption::LOG_PID,
            syslog::default_level_mapping,
            log::LevelFilter::Info,
            syslog::default_format,
        )?)
    }
}
