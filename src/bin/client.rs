use std::net::UdpSocket;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket");

    let hello_bytes = "hello".as_bytes();

    socket
        .send_to(hello_bytes, "127.0.0.1:1337")
        .expect("Failed to send data");

    println!("Sent {:#?} to the server", hello_bytes);
}
