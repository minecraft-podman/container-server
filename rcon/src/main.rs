use std::path::PathBuf;
use std::str::FromStr;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::process::exit;
use std::env;

extern crate rcon;


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <cmd>", args[0]);
        exit(1);
    }
    let cmd = &args[1];

    match read_properties(&match find_serverprops() {
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

enum PropLine {
    Comment,
    Prop(String, String)
}

enum PropParseError {
    NoValue
}

impl Into<io::Error> for PropParseError {
    fn into(self) -> io::Error {
        match self {
            PropParseError::NoValue => io::Error::new(io::ErrorKind::InvalidData, "No = found in line")
        }
    }
}

impl FromStr for PropLine {
    type Err = PropParseError;

    fn from_str(txt: &str) -> Result<Self, Self::Err> {
        if txt.starts_with("#") {
            return Ok(PropLine::Comment);
        }
        let bits: Vec<&str> = txt.splitn(2, '=').collect();
        if bits.len() != 2 {
            return Err(PropParseError::NoValue);
        }
        return Ok(PropLine::Prop(bits[0].to_string(), bits[1].to_string()))
    }
}

fn read_properties(path: &Path) -> io::Result<(u16, String)>  {
    let mut port: u16 = 0;
    let mut auth: String = "".to_string();
    for line in io::BufReader::new(File::open(path)?).lines().map(
        |l| -> io::Result<PropLine> { match l?.parse::<PropLine>() {
        Ok(pl) => Ok(pl),
        Err(err) => Err(err.into()),
    }}) {
        match line? {
            PropLine::Comment => {},
            PropLine::Prop(k, v) => match k.as_ref() {
                "rcon.port" => {
                    if let Ok(num) = v.parse() {
                        port = num;
                    }
                },
                "rcon.password" => {
                    auth = v;
                }
                &_ => {}
            }
        }
    }
    Ok((port, auth))
}


fn find_serverprops() -> Option<PathBuf> {
    // 1. Environment variable MINECRAFT_ROOT
    for (key, value) in env::vars() {
        if key == "MINECRAFT_ROOT" {
            return Some(PathBuf::from(value).join("server.properties"));
        }
    }
    // 2. The container's predefined root
    let container_root = Path::new("/mc"); // TODO: Can we read this from a compile flag?
    if container_root.exists() {
        return Some(container_root.join("server.properties"));
    }
    // Can't find it
    None
}
