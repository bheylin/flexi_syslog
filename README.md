A [flexi-logger](https://docs.rs/flexi_logger/0.22.0/flexi_logger/) [LogWriter](https://docs.rs/flexi_logger/0.22.0/flexi_logger/writers/trait.LogWriter.html) that writes to [syslog](https://datatracker.ietf.org/doc/html/rfc5424) on the Unix family of operating systems.

# Example Usage

```rust
use flexi_syslog::exe_name_from_env;

use flexi_logger::Logger;

fn main() -> anyhow::Result<()> {
    use flexi_syslog as syslog;

    let ident = exe_name_from_env()?;
    let syslog_writer = syslog::LibcWriter::try_new(
        &ident,
        syslog::Facility::Local0,
        syslog::LogOption::LOG_CONS | syslog::LogOption::LOG_PID,
        syslog::default_level_mapping,
        log::LevelFilter::Info,
        syslog::default_format,
    )?;

    let logger = Logger::try_with_str("info")?.log_to_writer(Box::new(syslog_writer));
    logger.start()?;

    log::info!("Info gets through");
    log::trace!("Trace is filtered");

    Ok(())
}
```

The writer only supports libc for now.