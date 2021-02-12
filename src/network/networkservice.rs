use std::{io, net::SocketAddr, sync::Arc};

use super::packets::{Packet, PacketData};


#[derive(Debug, Clone)]
pub struct AddressPacket {
    pub packet: Packet,
    pub address: SocketAddr,
}

pub trait Socket : Send + Sync {
    fn send_to(&self, buf: &[u8], addr: SocketAddr) -> io::Result<usize>;
    fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)>;
}

impl AddressPacket {
    pub fn new(packet: Packet, address: SocketAddr) -> AddressPacket {
        AddressPacket { packet, address }
    }
}

pub trait NetworkService {
    fn start(self: &Arc<Self>);
    fn get_packets(self: &Arc<Self>) -> Vec<AddressPacket>;
    fn queue_for_send(self: &Arc<Self>, packet_data: PacketData, to_addr: SocketAddr);
}
