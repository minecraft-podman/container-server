#![recursion_limit="256"]
use tokio::prelude::*;

use tokio::net::TcpStream;
use mcproto_min_async as mcp;
use std::path::Path;
use failure::Error;
use localmc::{find_serverprops, read_properties};


fn get_server_port(path: &Path) -> Result<u16, Error> {
    match
        read_properties(path)?.get("server-port").unwrap_or(&String::from("25565")).parse()
    {
        Ok(num) => Ok(num),
        Err(err) => Err(Error::from(err))
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let port = match get_server_port(&match find_serverprops() {
        Some(p) => p,
        None => {
            return Err(io::Error::new(io::ErrorKind::Other, "Unable to find server.properties"));
        }
    }) {
        Ok(p) => p,
        Err(e) => {
            return Err(io::Error::new(io::ErrorKind::Other, format!("{}", e)))
        }
    };


    let upstream = TcpStream::connect(format!("127.0.0.1:{}", port)).await?;
    let mut upstream = mcp::server::Client::create(upstream, mcp::protocol::Handshake);
    let p = mcp::protocol::handshake::Serverbound::ServerListPing {
        version: 498,
        host: "localhost".to_string(),
        port: 25567,
        next_state: mcp::protocol::ProtocolState::Status
    };
    upstream.write(&p).await?;
    let mut upstream = upstream.set_protocol(mcp::protocol::Status);
    upstream.write(&mcp::protocol::status::Serverbound::Request).await?;
    match upstream.read_cb().await? {
        mcp::protocol::status::Clientbound::ServerListResp { data } => 
            println!("{}", data),
        c => 
            panic!("Received an unexpected packet {:?}", c)
    }

    Ok(())
}
