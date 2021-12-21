mod test;

use flexi_logger::Logger;

#[test]
fn should_write_using_max_bytes() -> anyhow::Result<()> {
    let writer = test::default_builder()?.max_bytes(60).build()?;

    let logger = Logger::try_with_str("info")?.log_to_writer(Box::new(writer));
    logger.start()?;

    log::info!("Info gets through");
    log::trace!("Trace is filtered");
    log::info!("This line is not truncated");
    log::info!("This line is truncated here =====> as Builder::max_bytes is Some");

    Ok(())
}
