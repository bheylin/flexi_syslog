fn main() {
    let formatter = syslog::Formatter3164::default();
    let sys_logger = syslog::unix(formatter).expect("Failed to init unix socket");
    let writer = flexi_syslog::Builder::default()
        .max_bytes(60)
        .build(sys_logger);

    let logger = flexi_logger::Logger::try_with_str("info")
        .expect("Failed to build logger")
        .log_to_writer(Box::new(writer));

    logger.start().expect("Failed to start logger");

    log::info!("Info gets through");
    log::trace!("Trace is filtered");
    log::info!("This line is not truncated");
    log::info!("This line is truncated here =====> as Builder::max_bytes is Some");
}
