//! An implementation of a minecraft server packet parser. This module allows you to create and
//! handshake a client.

use futures::prelude::*;

//use tokio::prelude::*;
use tokio::io;
use tokio::net::TcpStream;

use tokio_util::codec::{FramedRead,FramedWrite};
use crate::codec::{RawPacket, Codec};
use crate::ser;
use crate::protocol::{self,*};


type ReadHalf = io::ReadHalf<TcpStream>;
type WriteHalf = io::WriteHalf<TcpStream>;

pub struct ClientReader<P>(pub FramedRead<ReadHalf, Codec>, pub P);
pub struct ClientWriter(pub FramedWrite<WriteHalf, Codec>);

impl<P> std::ops::Deref for ClientReader<P> {
    type Target = FramedRead<ReadHalf,Codec>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<P> std::ops::DerefMut for ClientReader<P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl std::ops::Deref for ClientWriter {
    type Target = FramedWrite<WriteHalf, Codec>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for ClientWriter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<P: Protocol> ClientReader<P> {
    pub async fn read(&mut self) -> Result<P::Serverbound,io::Error> {
        Ok(self.read_raw().await?.into_serverbound::<P>()?)
    }
    pub async fn read_raw(&mut self) -> Result<RawPacket,io::Error> {
        self.next().await.ok_or(io::ErrorKind::UnexpectedEof)?
    }
    pub fn set_protocol<P2: Protocol>(self, protocol: P2) -> ClientReader<P2> {
        ClientReader(self.0, protocol)
    }
}
impl ClientWriter {
    pub async fn write<S: Packet>(&mut self, s: &S) -> Result<(),io::Error> {
        let data = ser::to_bytes(&s);
        self.send(RawPacket(data)).await?;
        Ok(())
    }
    pub async fn write_raw(&mut self, data: &RawPacket) -> Result<(),io::Error> {
        self.send(data.clone()).await?;
        Ok(())
    }
}

pub enum Connection {
    Status(Client<Status>),
}

pub struct Client<P> {
    pub read:  ClientReader<P>,
    pub write: ClientWriter,
    version: i32,
    host: String,
    port: u16
}
impl Connection {
    /// Attempt to handshake a client.
    pub async fn new(s: TcpStream) -> Result<Connection, io::Error> {
        let mut client = Client::create(s, protocol::Handshake);

        let protocol::handshake::Serverbound::ServerListPing {
            next_state, version, host, port
        } = client.read().await?;
        client.version = version;
        client.host = host;
        client.port = port;
        if next_state == protocol::ProtocolState::Status {
            Ok(Connection::Status(client.set_protocol(Status)))
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "This library does not support logging in"))
        }
    }
}

#[derive(Default)]
pub struct ConnectionSettings {
    compression: Option<u32>
}

impl ConnectionSettings {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set_compression(mut self, threshold: u32) -> Self {
        self.compression = Some(threshold);
        self
    }
}

impl<P: Protocol> Client<P> {
    pub fn create(s: TcpStream, protocol: P) -> Self {
        let (read,write) = tokio::io::split(s);
        let read = ClientReader(FramedRead::new(read, Codec::new()), protocol);
        let write = ClientWriter(FramedWrite::new(write, Codec::new()));

        Client {
            read, write, version: 0, host: String::new(), port: 0
        }
    }
    pub async fn read(&mut self) -> Result<P::Serverbound, io::Error> {
        Ok(self.read.read_raw().await?.into_serverbound::<P>()?)
    }
    pub async fn read_cb(&mut self) -> Result<P::Clientbound, io::Error> {
        Ok(self.read.read_raw().await?.into_clientbound::<P>()?)
    }
    pub async fn read_raw(&mut self) -> Result<RawPacket, io::Error> {
        self.read.read_raw().await
    }
    pub async fn write<S: Packet>(&mut self, s: &S) -> Result<(),io::Error> {
        self.write.write(s).await
    }
    pub async fn write_raw(&mut self, data: &RawPacket) -> Result<(),io::Error> {
        self.write.write_raw(data).await
    }
    pub fn set_protocol<P2: Protocol>(self, protocol: P2) -> Client<P2> {
        let Self { read, write, version, host, port } = self;
        let read = read.set_protocol(protocol);
        Client { read, write, version, host, port }
    }
}
