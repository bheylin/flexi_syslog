mod test;

use std::os::unix::net::UnixDatagram;

fn main() {
    let (tx, rx) = UnixDatagram::pair().unwrap();
    let logger_handle = test::setup_log_writer(tx);

    log::info!("Info gets through");
    log::trace!("Trace is filtered");

    logger_handle.flush();

    assert!(test::recv_str(&rx).unwrap().ends_with("Info gets through"));
}
