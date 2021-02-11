use std::{io, net::SocketAddr};

use super::packets::{Packet, PacketData};


#[derive(Debug, Clone)]
pub struct AddressPacket {
    pub packet: Packet,
    pub address: SocketAddr,
}

pub trait SocketWrapper {
    fn send_to(&self, buf: &[u8], addr: SocketAddr) -> io::Result<usize>;
    fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)>;
}

impl AddressPacket {
    pub fn new(packet: Packet, address: SocketAddr) -> AddressPacket {
        AddressPacket { packet, address }
    }
}

pub trait NetworkService {
    fn get_packets(&mut self) -> Vec<AddressPacket>;
    fn queue_for_send(&mut self, packet_data: PacketData, to_addr: SocketAddr);
}
