use std::net::UdpSocket;

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:1337").expect("Failed to bind socket");

    let mut buffer = [0; 1024];

    let (size, address) = socket
        .recv_from(&mut buffer)
        .expect("Failed to receive data");

    println!("We got {} bytes from {:?}", size, address);
}
