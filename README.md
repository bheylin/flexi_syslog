Flexi-Syslog
============

![docs](https://img.shields.io/badge/docs.rs-flexi-syslog?style=for-the-badge&labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64)
![crates](https://img.shields.io/crates/v/flexi-syslog.svg?style=for-the-badge&color=fc8d62&logo=rust)
![build status](https://img.shields.io/github/actions/workflow/status/bheylin/syslog-suite/ci.yml?logo=github&style=for-the-badge)


A [flexi-logger](https://docs.rs/flexi_logger/latest/flexi_logger/) [LogWriter](https://docs.rs/flexi_logger/latest/flexi_logger/writers/trait.LogWriter.html) that formats and transports log records to the syslog using the [syslog](https://docs.rs/syslog/6.0.1/syslog/index.html) crate.

```toml
[dependencies]
flexi_logger = "0.24"
flexi_syslog = "0.5"
syslog = "6.0"
```

# Example Usage

```rust
fn main() {
    // syslog's Formatter5424 does not implement the rfc5424 timestamp correctly
    let formatter = flexi_syslog::Formatter5424 {
        facility: syslog::Facility::LOG_USER,
        hostname: None,
        process: "basic".into(),
        pid: 0,
    };

    let sys_logger = syslog::unix(formatter).expect("Failed to init unix socket");

    let syslog_writer = flexi_syslog::log_writer::Builder::default()
        .max_log_level(log::LevelFilter::Info)
        .build(sys_logger);

    let logger = flexi_logger::Logger::try_with_str("info")
        .expect("Failed to init logger")
        .log_to_writer(Box::new(syslog_writer));

    let handle = logger.start().expect("Failed to start logger");

    log::info!("Info gets through");
    log::trace!("Trace is filtered");

    handle.flush();
}
```
