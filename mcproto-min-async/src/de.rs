///! An implementation of the minecraft protocol as a `serde` deserializer.

use std::io::Cursor;
use std::io::Read;
use std::io;

use std::fmt;

use serde::de::{self, DeserializeSeed, SeqAccess, Visitor};
use byteorder::{ReadBytesExt, BE};

use crate::varint;

pub fn from_slice<'de, T: de::Deserialize<'de>>(bytes: &'de [u8]) -> Result<T,Error> {
    let mut de = Deserializer::from_slice(bytes);
    T::deserialize(&mut de)
}

pub struct Deserializer<'de> {
    data: Cursor<&'de [u8]>
}

#[derive(Debug)]
pub enum Error {
    Any,
    PacketLength,
    Other,
    Custom(String)
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for Error {}
impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

impl From<io::Error> for Error {
    fn from(_e: io::Error) -> Error {
        Error::PacketLength
    }
}
impl From<io::ErrorKind> for Error {
    fn from(_e: io::ErrorKind) -> Error {
        Error::PacketLength
    }
}

impl From<Error> for io::Error {
    fn from(e: Error) -> io::Error {
        io::Error::new(io::ErrorKind::Other, e)
    }
}

impl<'de> Deserializer<'de> {
    pub fn from_slice(input: &'de [u8]) -> Deserializer<'de> {
        Deserializer { data: Cursor::new(input) }
    }
}
macro_rules! build_de {
    ($self:ident, $visitor:ident: $($f:ident => $e:expr),*; $($f2:ident($($args:tt)*) => $e2:expr),*) => {
        #[allow(unused_mut,unused)]
        impl<'de, 'a> serde::Deserializer<'de> for &'a mut Deserializer<'de> {
            type Error = Error;
            $(fn $f<V: Visitor<'de>>(mut $self, $visitor: V) -> Result<V::Value, Self::Error> { $e })*
            $(fn $f2<V: Visitor<'de>>(mut $self, $($args)*, $visitor: V) -> Result<V::Value, Self::Error> { $e2 })*
        }
    }
}

build_de! {
    self, visitor:
    deserialize_any => Err(Error::Any),
    deserialize_bool => visitor.visit_bool({
        match self.data.read_u8()? {
            0 => false,
            1 => true,
            _ => Err(Error::Other)?
        }
    }),
    deserialize_i8 => visitor.visit_i8(self.data.read_i8()?),
    deserialize_u8 => visitor.visit_u8(self.data.read_u8()?),
    deserialize_i16 => visitor.visit_i16(self.data.read_i16::<BE>()?),
    deserialize_u16 => visitor.visit_u16(self.data.read_u16::<BE>()?),
    deserialize_i32 => visitor.visit_i32(self.data.read_i32::<BE>()?),
    deserialize_u32 => visitor.visit_u32(self.data.read_u32::<BE>()?),
    deserialize_i64 => visitor.visit_i64(self.data.read_i64::<BE>()?),
    deserialize_u64 => visitor.visit_u64(self.data.read_u64::<BE>()?),
    deserialize_i128 => visitor.visit_i128(self.data.read_i128::<BE>()?),
    deserialize_u128 => visitor.visit_u128(self.data.read_u128::<BE>()?),
    deserialize_f32 => visitor.visit_f32(self.data.read_f32::<BE>()?),
    deserialize_f64 => visitor.visit_f64(self.data.read_f64::<BE>()?),
    // TODO
    deserialize_str => Err(Error::Any),
    deserialize_string => visitor.visit_string({
        let len = varint::read(&mut self.data)?;
        // TODO: take a slice of the string as is instead of allocating
        let mut buf = vec![0;len as usize];
        self.data.read_exact(&mut buf);

        String::from_utf8_lossy(&*buf).to_string()
    }),
    deserialize_char => Err(Error::Any),
    deserialize_bytes => {
        let len = varint::read(&mut self.data)?;
        let mut buf = vec![0;len as usize];
        self.data.read_exact(&mut buf);
        visitor.visit_bytes(&buf)
    },
    deserialize_byte_buf => {
        let len = varint::read(&mut self.data)?;
        let mut buf = vec![0;len as usize];
        self.data.read_exact(&mut buf);
        visitor.visit_byte_buf(buf)
    },
    deserialize_option => {
        match self.data.read_u8()? {
            0 => visitor.visit_none(),
            1 => visitor.visit_some(self),
            _ => Err(Error::Other)?
        }
    },
    deserialize_unit => Err(Error::Any),
    deserialize_map => Err(Error::Any),
    deserialize_identifier => Err(Error::Any),
    deserialize_ignored_any => Err(Error::Any),
    deserialize_seq => {
        let len = varint::read(&mut self.data)?;
        self.deserialize_tuple(len as usize, visitor)
    };
    deserialize_newtype_struct(name: &'static str) => {
        match name {
            "*VARINT" => {
                let num = varint::read(&mut self.data)?;
                visitor.visit_i32(num)
            },
            "*REST" => {
                let position = self.data.position() as usize;
                visitor.visit_bytes(&self.data.get_ref()[position..])
            },
            "*INTPREFIXED" => {
                let len = self.data.read_i32::<BE>()? as usize;
                visitor.visit_seq(Consequent { de: &mut self, len })
            },
            "*SHORTPREFIXED" => {
                let len = self.data.read_i16::<BE>()? as usize;
                visitor.visit_seq(Consequent { de: &mut self, len })
            },
            _ => visitor.visit_newtype_struct(self)
        }
    },
    deserialize_unit_struct(name: &'static str) => visitor.visit_unit(),
    deserialize_tuple_struct(name: &'static str, len: usize) => Err(Error::Any),
    deserialize_tuple(len: usize) => visitor.visit_seq(Consequent { de: &mut self, len }),
    deserialize_struct(name: &'static str, fields: &'static [&'static str]) => {
        self.deserialize_tuple(fields.len(), visitor)
    },
    deserialize_enum(name: &'static str, variants: &'static [&'static str]) => {
        visitor.visit_enum(Enum { de: &mut self })
    }
}

struct Consequent<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    len: usize
}

impl<'a,'de> SeqAccess<'de> for Consequent<'a, 'de> {
    type Error = Error;
    fn next_element_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>,Self::Error> {
        if self.len == 0 { return Ok(None) }
        self.len -= 1;
        let value = serde::de::DeserializeSeed::deserialize(seed, &mut *self.de)?;
        Ok(Some(value))
    }
    fn size_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}

struct Enum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>
}
impl<'a,'de> de::EnumAccess<'de> for Enum<'a, 'de> {
    type Error = Error;
    type Variant = Enum<'a,'de>;

    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error> {
        use serde::de::IntoDeserializer;
        let c = varint::read(&mut self.de.data)?;
        let val: Result<_,Self::Error> = seed.deserialize((c as u32).into_deserializer());
        Ok((val?, self))
    }
}

impl<'a,'de> de::VariantAccess<'de> for Enum<'a,'de> {
    type Error = Error;
    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value, Self::Error> {
        seed.deserialize(self.de)
    }
    fn tuple_variant<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error> {
        use serde::de::Deserializer;
        self.de.deserialize_tuple(len, visitor)
    }
    fn struct_variant<V: Visitor<'de>>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error> {
        use serde::de::Deserializer;
        self.de.deserialize_tuple(fields.len(), visitor)
    }
}
