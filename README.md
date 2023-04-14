Flexi-Syslog
============

![docs](https://img.shields.io/badge/docs.rs-flexi-syslog?style=for-the-badge&labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64)
![crates](https://img.shields.io/crates/v/flexi-syslog.svg?style=for-the-badge&color=fc8d62&logo=rust)
![build status](https://img.shields.io/github/actions/workflow/status/bheylin/syslog-suite/ci.yml?logo=github&style=for-the-badge)


A [flexi-logger](https://docs.rs/flexi_logger/latest/flexi_logger/) [LogWriter](https://docs.rs/flexi_logger/latest/flexi_logger/writers/trait.LogWriter.html) that formats and transports log records to the syslog using the [syslog](https://docs.rs/syslog/6.0.1/syslog/index.html) crate.

```toml
[dependencies]
flexi_logger = "0.25"
flexi_syslog = "0.6"
```

# Example Usage

```rust
use flexi_syslog::{default_level_mapping, net, v5424, BufferWriteErrorStrategy, LogWriter};

fn main() {
    let formatter = v5424::Formatter::new(
        syslog_fmt::Facility::User,
        "app.domain.com",
        "app_test",
        None,
    );

    let socket = net::unix::any_recommended_socket().expect("Failed to init unix socket");
    let transport = socket.try_into().unwrap();

    let syslog_writer = LogWriter::<1024, _>::new(
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
```
