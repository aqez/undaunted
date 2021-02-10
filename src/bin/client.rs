use std::net::UdpSocket;

use undaunted::network::packets::*;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket");

    loop {
        let mut buffer = String::new();
        std::io::stdin()
            .read_line(&mut buffer)
            .expect("Failed to get buffer?");

        let packet = Packet::new(1, PacketData::Talk(TalkData::new(buffer)));
        dbg!(&packet);

        let bytes = rmp_serde::to_vec(&packet).expect("Failed to serialize");

        socket
            .send_to(&bytes, "127.0.0.1:1337")
            .expect("Failed to send data");

        println!("Sent {:#?} to the server", bytes);
    }
}
