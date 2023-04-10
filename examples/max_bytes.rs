use std::sync::{Arc, Mutex};

fn main() {
    let formatter = syslog_fmt::v5424::Formatter::with_max_bytes(
        syslog_fmt::Facility::User,
        "app.domain.com",
        "app_test",
        None,
        50,
    );

    let socket = syslog_net::unix::any_recommended_socket().expect("Failed to init unix socket");
    let socket_writer = syslog_net::unix::SocketWriter { socket };

    let syslog_writer = flexi_syslog::LogWriter {
        backend: Arc::new(Mutex::new(socket_writer)),
        formatter,
        max_log_level: log::LevelFilter::Info,
        format_fn: flexi_syslog::default_format,
        level_to_severity: flexi_syslog::default_level_mapping,
    };

    let logger = flexi_logger::Logger::try_with_str("info")
        .expect("Failed to init logger")
        .log_to_writer(Box::new(syslog_writer));

    let handle = logger.start().expect("Failed to start logger");

    log::info!("Info gets through");
    log::trace!("Trace is filtered");
    log::info!("This line is not truncated");
    log::info!("This line is truncated here =><= as log_writer::Builder::max_bytes is Some");

    handle.flush();
}
