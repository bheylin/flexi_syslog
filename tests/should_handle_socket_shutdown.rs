mod test;

use std::{io, os::unix::net::UnixDatagram};

#[allow(unsafe_code)]
fn main() {
    let socket_path = test::socket_path();

    // setup a socket that represents the receiving syslog daemon
    let rx = UnixDatagram::bind(&socket_path).unwrap();
    rx.set_nonblocking(true).unwrap();

    // setup a socket to transmit data to the syslog daemon
    let tx = UnixDatagram::unbound().unwrap();
    tx.connect(&socket_path).unwrap();

    let logger_handle = test::setup_log_writer(tx.try_clone().unwrap());

    let first_message = "Info gets through";
    let second_message = "Second info gets through";
    log::info!("{first_message}");
    log::info!("{second_message}");

    logger_handle.flush();

    assert!(test::recv_str(&rx).unwrap().ends_with(first_message));
    assert!(test::recv_str(&rx).unwrap().ends_with(second_message));

    drop(rx);
    std::fs::remove_file(&socket_path).unwrap();

    log::info!("Third info is lost due to shudown socket (BrokenPipe)");
    logger_handle.flush();

    let rx = UnixDatagram::bind(&socket_path).unwrap();
    rx.set_nonblocking(true).unwrap();

    println!("rx reconnected");
    assert_eq!(
        test::recv_str(&rx).unwrap_err().kind(),
        io::ErrorKind::WouldBlock,
        "Socket is set to non-blocking, it will Err instead of blocking"
    );

    let forth_message = "Forth info gets through, after socket reconnection";
    log::info!("{forth_message}");

    assert!(test::recv_str(&rx).unwrap().ends_with(forth_message));
}
