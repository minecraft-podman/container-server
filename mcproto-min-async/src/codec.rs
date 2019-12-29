use tokio::io;
use tokio_util::codec::{Encoder, Decoder};

use flate2::{Compression, write::ZlibEncoder, read::ZlibDecoder};

use bytes::{Bytes, BytesMut, BufMut};

use crate::{de, varint, protocol::Protocol};

// TODO: warn that you can't use it for both

#[derive(Default)]
pub struct Codec {
    compression: Option<i32>,
}

impl Codec {
    pub fn new() -> Self {
        Self::default()
    }
    /// Enables compression with the specified threshold. It is not advisable to re-specify the
    /// compression limit.
    pub fn set_compression(&mut self, limit: i32) {
        self.compression = Some(limit);
    }
}

impl Decoder for Codec {
    type Item = RawPacket;
    type Error = io::Error;
    fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<RawPacket>> {
        let buf = src;
        let (len, off) = if let Some(c) = varint::read_slice(&buf[..]) { c }
            else { return Ok(None) };
        if buf.len() < len as usize + off {
            return Ok(None)
        }
        let mut data = buf.split_to(len as usize + off).split_off(off).freeze();
        let data = if self.compression.is_some() {
            let (compressed_length,offset) = varint::read_slice(&data[..]).ok_or(io::ErrorKind::Other)?;
            if compressed_length == 0 {
                data.split_off(1)
            } else {
                use std::io::Read;
                let mut r = ZlibDecoder::new(&data[offset..]);
                let mut out = Vec::new();
                r.read_to_end(&mut out).map_err(|c| {println!("Decompression error: {} ({:X?})", c, data); c})?;
                bytes::Bytes::from(out)
            }
        } else {
            data
        };
        Ok(Some(RawPacket(data)))
    }
}

impl Encoder for Codec {
    type Item = RawPacket;
    type Error = io::Error;
    fn encode(&mut self, item: RawPacket, buf: &mut BytesMut) -> Result<(), io::Error> {
        let compression = self.compression;
        let write_packet = |item: Bytes, target: &mut BytesMut| {
            match compression {
                Some(c) if item.len() >= c as usize => {
                    use std::io::Write;
                    let compressed = Vec::new();
                    let ulen = varint::encode(item.len() as i32);
                    let mut compressor = ZlibEncoder::new(compressed, Compression::default());
                    compressor.write_all(&item).unwrap();
                    let compressed = compressor.finish().unwrap();
                    let len = varint::encode((ulen.len() + compressed.len()) as i32);
                    target.reserve(len.len() + ulen.len() + compressed.len());
                    target.put(&*len);
                    target.put(&*ulen);
                    target.put(&*compressed);
                }
                Some(_) => {
                    let len = varint::encode(item.len() as i32 + 1);
                    let zero = varint::encode(0);
                    target.reserve(len.len() + 1 + item.len());
                    target.put(&*len);
                    target.put(&*zero);
                    target.put(item);
                }
                None => {
                    let len = varint::encode(item.len() as i32);
                    target.reserve(len.len() + item.len());
                    target.put(&*len);
                    target.put(item);
                }
            }
        };
        write_packet(item.0, buf);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RawPacket(pub Bytes);

impl RawPacket {
    pub fn into_serverbound<P: Protocol>(&self) -> Result<P::Serverbound, de::Error> {
        de::from_slice(&self.0)
    }
    pub fn into_clientbound<P: Protocol>(&self) -> Result<P::Clientbound, de::Error> {
        de::from_slice(&self.0)
    }
}

impl std::ops::Deref for RawPacket {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &*self.0
    }
}
