use std::net::UdpSocket;

use undaunted::network::{NetworkService, UdpNetworkService, packets::*};

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket");


    let network_service = UdpNetworkService::new(socket);
    network_service.start();
    let server_address = "127.0.0.1:1337".parse().unwrap();

    loop {
        let mut buffer = String::new();
        std::io::stdin()
            .read_line(&mut buffer)
            .expect("Failed to get buffer?");

        network_service.queue_for_send(PacketData::Talk(TalkData::new(buffer)), server_address);
    }
}
