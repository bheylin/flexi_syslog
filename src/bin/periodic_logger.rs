use std::time::Duration;

use flexi_syslog::{net, v5424, BufferWriteErrorStrategy};

fn main() {
    let formatter = v5424::Formatter::from_config(v5424::Config {
        facility: syslog_fmt::Facility::User,
        hostname: Some("app.domain.com"),
        app_name: Some("app_test"),
        proc_id: None,
    });

    let tx =
        net::unix::any_recommended_datagram_socket().expect("failed to find a valid syslog socket");
    let transport = tx.try_into().unwrap();

    let syslog_writer = flexi_syslog::LogWriter::<1024, _>::new(
        formatter,
        transport,
        net::reconnect::AcquireSame::new(),
        log::LevelFilter::Info,
        flexi_syslog::default_level_mapping,
        BufferWriteErrorStrategy::Ignore,
    );

    let logger = flexi_logger::Logger::try_with_str("info")
        .expect("Failed to init logger")
        .log_to_writer(Box::new(syslog_writer));

    let handle = logger.start().unwrap();

    let mut i = 0;
    loop {
        i += 1;
        log::info!("Periodic log message #{i}");
        handle.flush();
        std::thread::sleep(Duration::from_secs(1));
    }
}
