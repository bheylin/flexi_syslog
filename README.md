Flexi-Syslog
============

[![docs][docs-badge]][docs-url]
[![crates][crates-badge]][crates-url]
[![build status][actions-badge]][actions-url]

[docs-badge]: https://img.shields.io/docsrs/flexi_syslog
[docs-url]: https://docs.rs/flexi_logger/latest/flexi_logger
[crates-badge]: https://img.shields.io/crates/v/flexi_syslog
[crates-url]: https://crates.io/crates/flexi_logger
[actions-badge]: https://github.com/bheylin/flexi_syslog/workflows/CI/badge.svg
[actions-url]: https://github.com/bheylin/flexi_syslog/actions?query=workflow%3ACI+branch%3Amain

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
