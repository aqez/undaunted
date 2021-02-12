use std::{
    net::{SocketAddr, UdpSocket},
    sync::{Arc, Mutex},
};

use super::{
    packets::{Packet, PacketData},
    AddressPacket, NetworkService, Socket,
};

pub struct UdpNetworkService<TSocket: Socket> {
    socket: TSocket,
    received_packets: Mutex<Vec<AddressPacket>>,
    to_send: Mutex<Vec<AddressPacket>>,
}

impl<TSocket: Socket> UdpNetworkService<TSocket> {
    pub fn new(socket: TSocket) -> Arc<Self> {
        Arc::new(Self {
            socket,
            received_packets: Mutex::new(Vec::new()),
            to_send: Mutex::new(Vec::new()),
        })
    }

    fn send_packets(self: &Arc<Self>) {
        let copy = {
            let mut to_send = self.to_send.lock().unwrap();
            println!("Copying {} packets", to_send.len());

            let copy = to_send.to_vec();
            to_send.clear();

            copy
        };

        for ap in copy.iter() {
            let bytes = rmp_serde::to_vec(&ap.packet).expect("Failed to serialize");
            println!("Trying to send {} bytes to {:?}", bytes.len(), ap.address);
            self.socket
                .send_to(&bytes, ap.address)
                .expect("Failed to send data");
        }
    }

    fn receive_packets(self: &Arc<Self>) {
        let mut buffer = [0; 1024];
        let (size, address) = self
            .socket
            .recv_from(&mut buffer)
            .expect("Failed to receive data");

        let packet: Packet =
            rmp_serde::from_read_ref(&buffer[0..size]).expect("Failed to deserialize");

        let address_packet = AddressPacket::new(packet, address);
        let mut received = self.received_packets.lock().unwrap();
        received.push(address_packet);
    }
}

impl<TSocket: Socket + 'static> NetworkService for UdpNetworkService<TSocket> {
    fn get_packets(self: &Arc<Self>) -> Vec<AddressPacket> {
        let copy = {
            let mut received = self.received_packets.lock().unwrap();
            let copy = received.to_vec();
            received.clear();

            copy
        };
        copy
    }

    fn queue_for_send(self: &Arc<Self>, packet_data: PacketData, to_addr: SocketAddr) {
        let packet = Packet::new(1, packet_data);
        let address_packet = AddressPacket::new(packet, to_addr);
        self.to_send.lock().unwrap().push(address_packet);
    }

    fn start(self: &Arc<Self>) {
        std::thread::spawn({
            let clone = Arc::clone(self);
            move || loop {
                clone.receive_packets();
                std::thread::sleep_ms(500);
            }
        });

        std::thread::spawn({
            let clone = Arc::clone(self);
            move || loop {
                clone.send_packets();
                std::thread::sleep_ms(500);
            }
        });
    }
}

impl Socket for UdpSocket {
    fn send_to(&self, buf: &[u8], addr: SocketAddr) -> std::io::Result<usize> {
        self.send_to(buf, addr)
    }

    fn recv_from(&self, buf: &mut [u8]) -> std::io::Result<(usize, SocketAddr)> {
        self.recv_from(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NullSocket;
    impl Socket for NullSocket {
        fn send_to(&self, buf: &[u8], addr: SocketAddr) -> std::io::Result<usize> {
            todo!()
        }

        fn recv_from(&self, buf: &mut [u8]) -> std::io::Result<(usize, SocketAddr)> {
            todo!()
        }
    }

    #[test]
    fn queue_for_send_adds_to_vec_to_send() {
        let network = UdpNetworkService::new(NullSocket {});
        network.queue_for_send(PacketData::Ack(1), "127.0.0.1:0".parse().unwrap());

        assert_eq!(1, network.to_send.lock().unwrap().len());
    }

    #[test]
    fn get_packets_gives_back_correct_number_of_packets() {
        let network = UdpNetworkService::new(NullSocket {});
        {
            let mut received = network.received_packets.lock().unwrap();
            received.push(AddressPacket::new(
                Packet::new(0, PacketData::Ack(1)),
                "127.0.0.1:0".parse().unwrap(),
            ));
        }

        let received = network.get_packets();

        assert_eq!(1, received.len());
    }

    #[test]
    fn get_packets_clears_old_packets() {
        let mut network = UdpNetworkService::new(NullSocket {});
        {
            let mut received = network.received_packets.lock().unwrap();
            received.push(AddressPacket::new(
                Packet::new(0, PacketData::Ack(1)),
                "127.0.0.1:0".parse().unwrap(),
            ));
        }

        let _ = network.get_packets();

        let received = network.received_packets.lock().unwrap();
        assert_eq!(0, received.len());
    }
}
