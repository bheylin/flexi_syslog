mod test;

use flexi_logger::Logger;

#[test]
fn should_write_using_default_config() -> anyhow::Result<()> {
    let writer = test::default_builder()?.build()?;
    let logger = Logger::try_with_str("info")?.log_to_writer(Box::new(writer));
    logger.start()?;

    log::info!("Info gets through");
    log::trace!("Trace is filtered");

    Ok(())
}
