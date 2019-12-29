//! Minecraft style varints and varlongs.

use std::io::{self, Read};
use serde::{Serialize, Serializer, Deserialize, Deserializer, de};
use byteorder::{ReadBytesExt};
//use smallvec::{smallvec, SmallVec};

/// Helper wrapper to deserialize and serialize as varint automatically.
#[derive(Debug)]
pub struct VarInt(pub i32);

impl<'de> Deserialize<'de> for VarInt {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<VarInt, D::Error> {
        deserialize(de).map(|c| VarInt(c))
    }
}

impl Serialize for VarInt {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_newtype_struct("*VARINT", &self.0)
    }
}


pub fn deserialize<'de, D: Deserializer<'de>>(de: D) -> Result<i32, D::Error> {
    struct VarIntVisitor;
    impl<'de> de::Visitor<'de> for VarIntVisitor {
        type Value = i32;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("varint")
        }
        fn visit_i32<E: de::Error>(self, value: i32) -> Result<Self::Value, E> {
            Ok(value)
        }
    }
    de.deserialize_newtype_struct("*VARINT", VarIntVisitor)
}
pub fn serialize<S: Serializer>(v: &i32, ser: S) -> Result<S::Ok, S::Error> {
    ser.serialize_newtype_struct("*VARINT", v)
}

pub fn read_slice(src: &[u8]) -> Option<(i32,usize)> {
    let mut len = 0;
    let read = || { len += 1; src.get(len-1).cloned().ok_or(()) };
    decode_varint(read, ()).ok().map(|c| (c, len))
}

pub fn read(mut src: impl Read) -> io::Result<i32> {
    decode_varint(|| { src.read_u8() }, io::Error::new(io::ErrorKind::Other, "incorrect varint"))
}

fn decode_varint<E>(mut read: impl FnMut() -> Result<u8,E>, err: E) -> Result<i32,E> {
    let mut radix: u64 = 128;
    let msb: u8 = 0b10000000; /* Only the MSB set */

    /* First we read the varint as an unsigned int */
    let mut buf = read()?;
    let mut res = (buf & (!msb)) as u64;

    let mut i: usize = 0;
    while (buf & msb) != 0 {
        if i >= 5 {
            return Err(err);
        }

        i += 1;
        buf = read()?;
        res += ((buf & (!msb)) as u64) * radix;
        radix <<= 7;
    }

    /* Convert to signed */
    if res >= 4294967296 {
        return Err(err);
    }

    if res > 2147483647 {
        return Ok((4294967296 - res) as i32 * -1);
    } else {
        return Ok(res as i32);
    }
}


pub fn encode(val: i32) -> Vec<u8> {
    if val == 0 {
        return vec![0];
    }

    let mut vec = Vec::new();

    let mut tmp = {
        if val > 0 {
            val as u32
        } else {
            let mut tmp = (val * -1) as u32;
            tmp ^= 4294967295;
            tmp + 1
        }
    };

    while tmp > 0 {
        /* By default we add the carrying bit to all bytes */
        vec.push(((tmp % 128) as u8) | (1 << 7));
        tmp = tmp >> 7;
    }

    /* and then we remove the carrying bit from the last byte */
    let i = vec.len() - 1;
    vec[i] &= !(1 << 7);

    vec
}
