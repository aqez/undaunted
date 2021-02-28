use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
    sync::{Arc, Mutex},
    time::Duration,
};

use chrono::{DateTime, Utc};

use super::{
    packets::{Packet, PacketData},
    AddressPacket, NetworkService, Socket,
};

struct SentPacket {
    address_packet: AddressPacket,
    sent_time: DateTime<Utc>,
}

impl SentPacket {
    fn new(address_packet: AddressPacket, sent_time: DateTime<Utc>) -> SentPacket {
        SentPacket {
            address_packet,
            sent_time,
        }
    }
}

pub struct UdpNetworkService<TSocket: Socket> {
    socket: TSocket,
    received_packets: Mutex<Vec<AddressPacket>>,
    to_send: Mutex<Vec<AddressPacket>>,
    unacked_packets: Mutex<Vec<SentPacket>>,
    packet_ids: Mutex<HashMap<SocketAddr, u32>>,
}

impl<TSocket: Socket + 'static> UdpNetworkService<TSocket> {
    pub fn new(socket: TSocket) -> Arc<Self> {
        Arc::new(Self {
            socket,
            received_packets: Mutex::new(Vec::new()),
            to_send: Mutex::new(Vec::new()),
            unacked_packets: Mutex::new(Vec::new()),
            packet_ids: Mutex::new(HashMap::new()),
        })
    }

    fn send_packets(self: &Arc<Self>) {
        let copy = {
            let mut to_send = self.to_send.lock().unwrap();
            let copy = to_send.to_vec();
            to_send.clear();
            copy
        };

        let mut unacked = self.unacked_packets.lock().unwrap();
        for ap in copy.into_iter() {
            let bytes = rmp_serde::to_vec(&ap.packet).expect("Failed to serialize");
            println!("Trying to send {} bytes to {:?}", bytes.len(), ap.address);
            self.socket
                .send_to(&bytes, ap.address)
                .expect("Failed to send data");

            match ap.packet.packet_data {
                PacketData::Ack(_) => {}
                _ => unacked.push(SentPacket::new(ap, Utc::now())),
            }
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

        match packet.packet_data {
            PacketData::Ack(_) => {
                self.remove_unacked_packet(packet.id, address);
                return;
            }
            _ => {
                self.queue_for_send(PacketData::Ack(packet.id), address);
            }
        }

        let address_packet = AddressPacket::new(packet, address);
        let mut received = self.received_packets.lock().unwrap();
        received.push(address_packet);
    }

    fn remove_unacked_packet(self: &Arc<Self>, id: u32, address: SocketAddr) {
        let mut i = 0;
        let mut unacked = self.unacked_packets.lock().unwrap();

        while i != unacked.len() {
            if unacked[i].address_packet.packet.id == id
                && unacked[i].address_packet.address == address
            {
                unacked.remove(i);
            } else {
                i += 1;
            }
        }
    }

    fn requeue_unacked_packets(self: &Arc<Self>) {
        let max_age = chrono::Duration::milliseconds(1000);
        let now = Utc::now();
        let mut to_send = self.to_send.lock().unwrap();
        let mut unacked = self.unacked_packets.lock().unwrap();

        let mut i = 0;
        while i != unacked.len() {
            if (now - unacked[i].sent_time) > max_age {
                to_send.push(unacked.remove(i).address_packet);
            } else {
                i += 1;
            }
        }
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
        let mut ids = self.packet_ids.lock().unwrap();

        let id = ids.entry(to_addr).or_insert(0);
        let packet = Packet::new(*id, packet_data);

        *id += 1;

        let address_packet = AddressPacket::new(packet, to_addr);
        self.to_send.lock().unwrap().push(address_packet);
    }

    fn start(self: &Arc<Self>) {
        std::thread::spawn({
            let clone = Arc::clone(self);
            move || loop {
                clone.receive_packets();
                std::thread::sleep(Duration::from_millis(5));
            }
        });

        std::thread::spawn({
            let clone = Arc::clone(self);
            move || loop {
                clone.requeue_unacked_packets();
                clone.send_packets();
                std::thread::sleep(Duration::from_millis(5));
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
    use crate::network::packets::*;

    struct NullSocket;
    impl Socket for NullSocket {
        fn send_to(&self, buf: &[u8], _addr: SocketAddr) -> std::io::Result<usize> {
            Ok(buf.len())
        }

        fn recv_from(&self, _buf: &mut [u8]) -> std::io::Result<(usize, SocketAddr)> {
            Ok((0, "127.0.0.1:0".parse().unwrap()))
        }
    }

    #[test]
    fn queue_for_send_adds_incremented_packet_ids_for_same_client_to_send_list() {
        let network = UdpNetworkService::new(NullSocket {});
        let to_addr = "127.0.0.1:0".parse().unwrap();
        network.queue_for_send(PacketData::Talk(TalkData::new("ha".to_string())), to_addr);
        network.queue_for_send(PacketData::Talk(TalkData::new("ha".to_string())), to_addr);

        let to_send = network.to_send.lock().unwrap();

        assert_eq!(0, to_send[0].packet.id);
        assert_eq!(1, to_send[1].packet.id);
    }

    #[test]
    fn queue_for_send_doesnt_increment_for_different_client_ids() {
        let network = UdpNetworkService::new(NullSocket {});
        network.queue_for_send(
            PacketData::Talk(TalkData::new("ha".to_string())),
            "127.0.0.1:10".parse().unwrap(),
        );
        network.queue_for_send(
            PacketData::Talk(TalkData::new("ha".to_string())),
            "127.0.0.1:11".parse().unwrap(),
        );
        network.queue_for_send(
            PacketData::Talk(TalkData::new("ha".to_string())),
            "127.0.0.1:10".parse().unwrap(),
        );
        network.queue_for_send(
            PacketData::Talk(TalkData::new("ha".to_string())),
            "127.0.0.1:11".parse().unwrap(),
        );

        let to_send = network.to_send.lock().unwrap();

        assert_eq!(0, to_send[0].packet.id);
        assert_eq!(0, to_send[1].packet.id);
        assert_eq!(1, to_send[2].packet.id);
        assert_eq!(1, to_send[3].packet.id);
    }

    #[test]
    fn send_packets_adds_sent_packets_to_unacked_list() {
        let network = UdpNetworkService::new(NullSocket {});
        network.queue_for_send(PacketData::Ack(1), "127.0.0.1:0".parse().unwrap());

        network.send_packets();

        assert_eq!(0, network.to_send.lock().unwrap().len());
        assert_eq!(1, network.unacked_packets.lock().unwrap().len());
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
        let network = UdpNetworkService::new(NullSocket {});
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
