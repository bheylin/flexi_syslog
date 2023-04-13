use std::os::unix::net::UnixDatagram;

use flexi_logger::LoggerHandle;

use flexi_syslog::{BrokenPipeErrorStrategy, FullBufferErrorStrategy};

fn main() {
    let (tx, rx) = UnixDatagram::pair().unwrap();
    let logger_handle = setup_log_writer(tx);

    log::info!("Info gets through");
    log::trace!("Trace is filtered");

    logger_handle.flush();

    let mut buf = vec![0u8; 128];
    let bytes_received = rx.recv(&mut buf).unwrap();
    buf.truncate(bytes_received);
    let s = String::from_utf8(buf).unwrap();
    assert!(s.ends_with("Info gets through"));
    assert!(bytes_received > 0);
}

fn setup_log_writer(tx: UnixDatagram) -> LoggerHandle {
    let formatter = syslog_fmt::v5424::Formatter::new(
        syslog_fmt::Facility::User,
        "app.domain.com",
        "app_test",
        None,
    );

    let syslog_writer = flexi_syslog::LogWriter::<1024>::new(
        formatter,
        tx.into(),
        log::LevelFilter::Info,
        flexi_syslog::default_level_mapping,
        FullBufferErrorStrategy::Ignore,
        BrokenPipeErrorStrategy::Ignore,
    );

    let logger = flexi_logger::Logger::try_with_str("info")
        .expect("Failed to init logger")
        .log_to_writer(Box::new(syslog_writer));

    logger.start().unwrap()
}
