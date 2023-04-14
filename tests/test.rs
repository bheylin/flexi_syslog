use std::{io, os::unix::net::UnixDatagram};

use flexi_logger::LoggerHandle;
use syslog_net::reconnect;

use flexi_syslog::BufferWriteErrorStrategy;

pub fn setup_log_writer(tx: UnixDatagram) -> LoggerHandle {
    let formatter = syslog_fmt::v5424::Formatter::new(
        syslog_fmt::Facility::User,
        "app.domain.com",
        "app_test",
        None,
    );

    let transport = tx.try_into().unwrap();

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

    logger.start().unwrap()
}

/// Create a socket path with a random string of chars postfixed to the filename.
/// Example: `/tmp/socket-a2bDe6`
#[allow(dead_code)]
pub fn socket_path() -> String {
    use rand::{distributions::Alphanumeric, Rng};

    let temp_dir = std::env::temp_dir();
    let socket_postfix: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();

    format!("{}/socket-{}", temp_dir.display(), socket_postfix)
}

pub fn recv_str(rx: &UnixDatagram) -> io::Result<String> {
    let mut buf = vec![0u8; 128];
    let bytes_received = rx.recv(&mut buf)?;
    buf.truncate(bytes_received);
    Ok(String::from_utf8(buf).unwrap())
}
