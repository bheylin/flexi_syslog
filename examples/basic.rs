use syslog_net::reconnect;

use flexi_syslog::BufferWriteErrorStrategy;

fn main() {
    let formatter = syslog_fmt::v5424::Formatter::new(
        syslog_fmt::Facility::User,
        "app.domain.com",
        "app_test",
        None,
    );

    let socket = syslog_net::unix::any_recommended_socket().expect("Failed to init unix socket");
    let transport = socket.try_into().unwrap();

    let syslog_writer = flexi_syslog::LogWriter::<1024, _>::new(
        formatter,
        transport,
        reconnect::AcquireSame::new(),
        log::LevelFilter::Info,
        flexi_syslog::default_level_mapping,
        BufferWriteErrorStrategy::Ignore,
    );

    let logger = flexi_logger::Logger::try_with_str("info")
        .expect("Failed to init logger")
        .log_to_writer(Box::new(syslog_writer));

    let handle = logger.start().expect("Failed to start logger");

    log::info!("Info gets through");
    log::trace!("Trace is filtered");

    handle.flush();
}
