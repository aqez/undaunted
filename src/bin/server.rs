use std::{net::UdpSocket, time::Duration};

use undaunted::network::{NetworkService, UdpNetworkService};

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:1337").expect("Failed to bind socket");

    let network_service = UdpNetworkService::new(socket);

    network_service.start();

    loop {
        let packets = network_service.get_packets();

        if packets.len() > 0 {
            dbg!(packets);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}
