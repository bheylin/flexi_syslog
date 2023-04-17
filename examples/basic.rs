use flexi_syslog::{default_level_mapping, net, v5424, BufferWriteErrorStrategy, LogWriter};

fn main() {
    const BUFFER_SIZE: usize = 1024;

    let formatter = v5424::Formatter::new(v5424::Config {
        facility: syslog_fmt::Facility::User,
        hostname: Some("app.domain.com"),
        app_name: Some("app_test"),
        proc_id: None,
    });

    let socket = net::unix::any_recommended_socket().expect("Failed to init unix socket");
    let transport = socket.try_into().unwrap();

    let syslog_writer = LogWriter::<BUFFER_SIZE, _>::new(
        formatter,
        transport,
        net::reconnect::AcquireSame::new(),
        log::LevelFilter::Info,
        default_level_mapping,
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
