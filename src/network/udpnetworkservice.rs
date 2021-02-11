use std::net::SocketAddr;

use super::{
    packets::{Packet, PacketData},
    AddressPacket, NetworkService, SocketWrapper,
};

pub struct UdpNetworkService<TSocket: SocketWrapper> {
    socket: TSocket,
    received_packets: Vec<AddressPacket>,
    to_send: Vec<AddressPacket>,
}

impl<TSocket: SocketWrapper> UdpNetworkService<TSocket> {
    pub fn new(socket: TSocket) -> Self {
        Self {
            socket,
            received_packets: Vec::new(),
            to_send: Vec::new(),
        }
    }
}

impl<TSocket: SocketWrapper> NetworkService for UdpNetworkService<TSocket> {
    fn get_packets(&mut self) -> Vec<AddressPacket> {
        let copy = {
            let copy = self.received_packets.to_vec();
            self.received_packets.clear();

            copy
        };
        copy
    }

    fn queue_for_send(&mut self, packet_data: PacketData, to_addr: SocketAddr) {
        let packet = Packet::new(1, packet_data);
        let address_packet = AddressPacket::new(packet, to_addr);
        self.to_send.push(address_packet);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NullSocketWrapper;
    impl SocketWrapper for NullSocketWrapper {
        fn send_to(&self, buf: &[u8], addr: SocketAddr) -> std::io::Result<usize> {
            todo!()
        }

        fn recv_from(&self, buf: &mut [u8]) -> std::io::Result<(usize, SocketAddr)> {
            todo!()
        }
    }

    #[test]
    fn queue_for_send_adds_to_vec_to_send() {
        let mut network = UdpNetworkService::new(NullSocketWrapper {});
        network.queue_for_send(PacketData::Ack(1), "127.0.0.1:0".parse().unwrap());

        assert_eq!(1, network.to_send.len());
    }

    #[test]
    fn get_packets_gives_back_correct_number_of_packets() {
        let mut network = UdpNetworkService::new(NullSocketWrapper {});
        network.received_packets.push(AddressPacket::new(
            Packet::new(0, PacketData::Ack(1)),
            "127.0.0.1:0".parse().unwrap(),
        ));

        let received = network.get_packets();

        assert_eq!(1, received.len());
    }

    #[test]
    fn get_packets_clears_old_packets() {
        let mut network = UdpNetworkService::new(NullSocketWrapper {});
        network.received_packets.push(AddressPacket::new(
            Packet::new(0, PacketData::Ack(1)),
            "127.0.0.1:0".parse().unwrap(),
        ));

        let _ = network.get_packets();

        assert_eq!(0, network.received_packets.len());
    }
}
