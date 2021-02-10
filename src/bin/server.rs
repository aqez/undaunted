use std::net::UdpSocket;

use undaunted::network::packets::*;

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:1337").expect("Failed to bind socket");

    let mut buffer = [0; 1024];

    loop {
        let (size, address) = socket
            .recv_from(&mut buffer)
            .expect("Failed to receive data");

        let packet: Packet =
            rmp_serde::from_read_ref(&buffer[0..size]).expect("Failed to deserialize");

        println!("We got {} bytes from {:?}, the data was: {:#?}", size, address, packet);
    }
}
