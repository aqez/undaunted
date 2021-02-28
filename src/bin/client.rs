use std::{net::UdpSocket, sync::Arc, time::Duration};

use undaunted::network::{packets::*, NetworkService, UdpNetworkService};

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket");

    let network_service = UdpNetworkService::new(socket);
    network_service.start();
    let server_address = "127.0.0.1:1337".parse().unwrap();

    std::thread::spawn({
        let clone = Arc::clone(&network_service);
        move || loop {
            let packets = clone.get_packets();

            if packets.len() > 0 {
                dbg!(packets);
            }

            std::thread::sleep(Duration::from_millis(1000));
        }
    });

    loop {
        let mut buffer = String::new();
        std::io::stdin()
            .read_line(&mut buffer)
            .expect("Failed to get buffer?");

        network_service.queue_for_send(PacketData::Talk(TalkData::new(buffer)), server_address);
    }
}
