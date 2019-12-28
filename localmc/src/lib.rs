// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         assert_eq!(2 + 2, 4);
//     }
// }

use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::env;

#[macro_use] extern crate failure;

use failure::Fallible;

enum PropLine {
    Comment,
    Prop(String, String)
}

#[derive(Debug, Fail)]
pub enum PropParseError {
    #[fail(display = "No = found in line")]
    NoValue
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

pub fn read_properties(path: &Path) -> Fallible<HashMap<String, String>>  {
    let mut props = HashMap::new();
    for line in io::BufReader::new(File::open(path)?).lines() {
        match line?.parse::<PropLine>()? {
            PropLine::Comment => {},
            PropLine::Prop(k, v) => {props.insert(k, v);}
        }
    }
    Ok(props)
}


pub fn find_root() -> Option<PathBuf> {
    // 1. Environment variable MINECRAFT_ROOT
    for (key, value) in env::vars() {
        if key == "MINECRAFT_ROOT" {
            return Some(PathBuf::from(value));
        }
    }
    // 2. The container's predefined root
    let container_root = Path::new("/mc"); // TODO: Can we read this from a compile flag?
    if container_root.exists() {
        return Some(container_root.to_path_buf());
    }
    // Can't find it
    None
}

pub fn find_serverprops() -> Option<PathBuf> {
    find_root().map(|p| p.join("server.properties"))
}
