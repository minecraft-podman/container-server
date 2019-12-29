#![recursion_limit="256"]
use tokio::prelude::*;

use tokio::net::{TcpStream};
use mcproto_min_async as mcp;


#[tokio::main]
async fn main() -> io::Result<()> {
    let upstream = TcpStream::connect("127.0.0.1:25565").await?;
    let mut upstream = mcp::server::Client::create(upstream, mcp::protocol::Handshake);
    let p = mcp::protocol::handshake::Serverbound::ServerListPing {
        version: 498,
        host: "localhost".to_string(),
        port: 25567,
        next_state: mcp::protocol::ProtocolState::Status
    };
    upstream.write(&p).await?;
    let mut upstream = upstream.set_protocol(mcp::protocol::Status);
    let p = mcp::protocol::status::Serverbound::Request;
    upstream.write(&p).await?;
    let c = upstream.read_cb().await?;
    println!("{:?}", c);

    Ok(())
}
