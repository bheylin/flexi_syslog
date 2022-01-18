fn main() {
    let formatter = syslog::Formatter5424 {
        facility: syslog::Facility::LOG_USER,
        hostname: None,
        process: "basic".into(),
        pid: 0,
    };

    let sys_logger = syslog::unix(formatter).expect("Failed to init unix socket");

    let syslog_writer = flexi_syslog::log_writer::Builder::default()
        .max_log_level(log::LevelFilter::Info)
        .build(sys_logger);

    let logger = flexi_logger::Logger::try_with_str("info")
        .expect("Failed to init logger")
        .log_to_writer(Box::new(syslog_writer));

    let handle = logger.start().expect("Failed to start logger");

    log::info!("Info gets through");
    log::trace!("Trace is filtered");

    handle.flush();
}
