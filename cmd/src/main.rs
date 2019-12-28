use std::path::Path;
use std::process::exit;
use std::env;
use failure::Error;
use localmc::{read_properties, find_serverprops};
use rcon;


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <cmd>", args[0]);
        exit(1);
    }
    let cmd = &args[1];

    match read_portauth(&match find_serverprops() {
        Some(p) => p,
        None => {
            eprintln!("Unable to find server.properties");
            exit(10);
        }
    }) {
        Ok((port, auth)) => {
            let address = format!("localhost:{}", port);
            match run_cmd(&address, &auth, &cmd) {
                Ok(msg) => {
                    print!("{}", msg);
                },
                Err(e) => {
                    eprintln!("Error running command: {}", e);
                    exit(30);
                }
            }
        },
        Err(err) => {
            eprintln!("Error reading server.properties: {}", err);
            exit(20);
        }
    }
}

fn run_cmd(addr: &str, auth: &str, cmd: &str) -> rcon::Result<String> {
    let mut conn = rcon::Connection::connect(addr, auth)?;
    conn.cmd(cmd)
}

fn read_portauth(path: &Path) -> Result<(u16, String), Error>  {
    let props = read_properties(path)?;
    Ok((
        props.get("rcon.port").unwrap_or(&String::from("0")).parse()?,
        props.get("rcon.password").unwrap_or(&String::from("")).to_string()
    ))
}
