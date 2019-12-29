use serde::{Deserialize, Serialize, de::DeserializeOwned};
use crate::varint;
use crate::codec::RawPacket;

pub trait Protocol {
    type Serverbound: Packet;
    type Clientbound: Packet;
    const ID: u32;
}

pub trait Packet: DeserializeOwned + Serialize {
    /*fn as_keepalive(&self) -> Option<u64>;
    fn keepalive(data: u64) -> Self;*/
    fn into_raw(&self) -> RawPacket {
        RawPacket(crate::ser::to_bytes(&self))
    }
}

impl Packet for () {}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum ProtocolState {
    Handshake,
    Status,
}

pub struct Handshake;
impl Protocol for Handshake {
    type Serverbound = handshake::Serverbound;
    type Clientbound = ();
    const ID: u32 = 0;
}
pub mod handshake {
    use super::*;
    #[derive(Debug, Deserialize, Serialize)]
    pub enum Serverbound {
        ServerListPing {
            #[serde(with="varint")]
            version: i32,
            host: String,
            port: u16,
            next_state: ProtocolState
        }
    }
    impl Packet for Serverbound {}
}

pub struct Status;
impl Protocol for Status {
    type Serverbound = status::Serverbound;
    type Clientbound = status::Clientbound;
    const ID: u32 = 0;
}
pub mod status {
    use super::*;
    #[derive(Debug, Deserialize, Serialize)]
    pub enum Clientbound {
        ServerListResp { data: String },
        Pong(u64)
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub enum Serverbound {
        Request,
        Ping(u64)
    }
    impl Packet for Clientbound {}
    impl Packet for Serverbound {}
}
