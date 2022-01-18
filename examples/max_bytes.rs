fn main() {
    let formatter = syslog::Formatter5424 {
        facility: syslog::Facility::LOG_USER,
        hostname: None,
        process: "basic".into(),
        pid: 0,
    };

    let sys_logger = syslog::unix(formatter).expect("Failed to init unix socket");
    let writer = flexi_syslog::log_writer::Builder::default()
        .max_bytes(50)
        .build(sys_logger);

    let logger = flexi_logger::Logger::try_with_str("info")
        .expect("Failed to build logger")
        .log_to_writer(Box::new(writer));

    let handle = logger.start().expect("Failed to start logger");

    log::info!("Info gets through");
    log::trace!("Trace is filtered");
    log::info!("This line is not truncated");
    log::info!("This line is truncated here =><= as log_writer::Builder::max_bytes is Some");

    handle.flush();
}
