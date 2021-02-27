use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Packet {
    pub id: u32,
    pub packet_data: PacketData,
}

impl Packet {
    pub fn new(id: u32, packet_data: PacketData) -> Packet {
        Packet { id, packet_data }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PacketData {
    Talk(TalkData),
    Ack(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TalkData {
    pub phrase: String,
}

impl TalkData {
    pub fn new(phrase: String) -> TalkData {
        TalkData { phrase }
    }
}
