use flexi_syslog::FullBufferErrorStrategy;

fn main() {
    let formatter = syslog_fmt::v5424::Formatter::new(
        syslog_fmt::Facility::User,
        "app.domain.com",
        "app_test",
        None,
    );

    let socket = syslog_net::unix::any_recommended_socket().expect("Failed to init unix socket");

    let syslog_writer = flexi_syslog::LogWriter::<1024>::new(
        formatter,
        socket.into(),
        log::LevelFilter::Info,
        flexi_syslog::default_level_mapping,
        FullBufferErrorStrategy::Ignore,
    );

    let logger = flexi_logger::Logger::try_with_str("info")
        .expect("Failed to init logger")
        .log_to_writer(Box::new(syslog_writer));

    let handle = logger.start().expect("Failed to start logger");

    log::info!("Info gets through");
    log::trace!("Trace is filtered");

    handle.flush();
}
