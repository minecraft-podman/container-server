
use serde::{ser, Serialize};

use std::io::{self, Write};
use std::fmt;

use byteorder::{WriteBytesExt,BE};

use crate::varint;

pub fn to_writer<T: Serialize>(w: impl Write, value: &T) {
    let mut ser = Serializer { w };
    value.serialize(&mut ser).unwrap();
}

pub fn to_bytes<T: Serialize>(value: &T) -> bytes::Bytes {
    let mut bytes = Vec::new();
    to_writer(&mut bytes, value);
    bytes::Bytes::from(bytes)
}

#[derive(Debug)]
pub enum Error {
    Any,
    Custom(String),
    Type(&'static str),
    Other
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad("uh")
    }
}
impl std::error::Error for Error {}
impl serde::ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}
impl From<io::Error> for Error {
    fn from(_e: io::Error) -> Error {
        Error::Other
    }
}

pub struct Serializer<W> {
    w: W,
}

impl<W: Write> Serializer<W> {
    pub fn new(w: W) -> Self { Self { w } }
}

macro_rules! build_ser {
    ([$($lt:tt)+] $struct:ty, $ser:ty => $self:ident, $v:ident: $($rest:tt)*) => {
        #[allow(unused_mut,unused)]
        impl<$($lt)+> ser::Serializer for $struct {
            type Ok = ();
            type Error = Error;
            type SerializeSeq = $ser;
            type SerializeTuple = $ser;
            type SerializeTupleStruct = $ser;
            type SerializeTupleVariant = $ser;
            type SerializeMap = $ser;
            type SerializeStruct = $ser;
            type SerializeStructVariant = $ser;
            build_ser!(@item $self, $v, $($rest)*);
        }
    };
    (@item $self:ident, $v:ident, $f:ident: $t:ty => $e:expr, $($rest:tt)*) => {
        fn $f($self, $v: $t) -> Result<Self::Ok, Error> { Ok($e?) }
        build_ser!(@item $self, $v, $($rest)*);
    };
    (@item $self:ident, $v:ident, $f:ident ($($args:tt)*) -> $ret:ty => $e:expr, $($rest:tt)*) => {
        fn $f ($self, $($args)*) -> Result<$ret, Error> { $e }
        build_ser!(@item $self, $v, $($rest)*);
    };
    (@item $self:ident, $v:ident, $f:ident [$($gen:tt)*] ($($args:tt)*) -> $ret:ty => $e:expr, $($rest:tt)*) => {
        fn $f <$($gen)*>($self, $($args)*) -> Result<$ret, Error> { $e }
        build_ser!(@item $self, $v, $($rest)*);
    };
    (@item $self:ident, $v:ident, ) => {};
}

build_ser! {
    ['a, W: Write] &'a mut Serializer<W>, Self => self, v:
    serialize_bool: bool => self.w.write_u8(if v { 1 } else { 0 }),
    serialize_i8: i8 => self.w.write_i8(v),
    serialize_u8: u8 => self.w.write_u8(v),
    serialize_i16: i16 => self.w.write_i16::<BE>(v),
    serialize_u16: u16 => self.w.write_u16::<BE>(v),
    serialize_i32: i32 => self.w.write_i32::<BE>(v),
    serialize_u32: u32 => self.w.write_u32::<BE>(v),
    serialize_i64: i64 => self.w.write_i64::<BE>(v),
    serialize_u64: u64 => self.w.write_u64::<BE>(v),
    serialize_i128: i128 => self.w.write_i128::<BE>(v),
    serialize_u128: u128 => self.w.write_u128::<BE>(v),
    serialize_f32: f32 => self.w.write_f32::<BE>(v),
    serialize_f64: f64 => self.w.write_f64::<BE>(v),
    serialize_char: char => Err(Error::Type("char")),
    serialize_str: &str => {
        self.w.write_all(&*varint::encode(v.len() as i32))?;
        self.w.write_all(v.as_bytes())
    },
    serialize_bytes: &[u8] => {
        self.w.write_all(&*varint::encode(v.len() as i32))?;
        self.w.write_all(v)
    },
    serialize_unit_struct: &'static str => Ok::<(),Error>(()),

    serialize_some[T: ?Sized + Serialize](v: &T) -> Self::Ok => {
        self.serialize_bool(true)?;
        v.serialize(self)
    },
    serialize_none() -> Self::Ok => self.serialize_bool(false),
    serialize_newtype_struct[T: ?Sized + Serialize](name: &'static str, v: &T) -> Self::Ok => {
        match name {
            "*VARINT" => {  // make sure that you can't do this by accident by adding an asterisk there
                ser::Serialize::serialize(v, &mut SpecialSerializer { w: self, mode: Mode::VarInt })
            },
            "*REST" => {
                ser::Serialize::serialize(v, &mut SpecialSerializer { w: self, mode: Mode::Rest })
            },
            "*INTPREFIXED" => {
                ser::Serialize::serialize(v, &mut SpecialSerializer { w: self, mode: Mode::IntPrefixed })
            }
            "*SHORTPREFIXED" => {
                ser::Serialize::serialize(v, &mut SpecialSerializer { w: self, mode: Mode::ShortPrefixed })
            }
            _ => ser::Serialize::serialize(v, self)
        }
    },
    serialize_seq(len: Option<usize>) -> Self::SerializeSeq => {
        self.w.write_all(&*varint::encode(len.ok_or(Error::Any)? as i32))?;
        Ok(self)
    },
    serialize_tuple(len: usize) -> Self::SerializeTuple => Ok(self),
    serialize_map(len: Option<usize>) -> Self::SerializeMap => Ok(self),
    serialize_struct(_name: &'static str, _len: usize) -> Self::SerializeStruct => Ok(self),
    serialize_unit() -> Self::Ok => Ok(()),
    serialize_unit_variant(name: &'static str, variant_index: u32, _variant: &'static str) -> Self::Ok => {
        self.w.write_all(&*varint::encode(variant_index as i32))?;
        Ok(())
    },
    serialize_newtype_variant[T: ?Sized+Serialize](name: &'static str, variant_index: u32, _variant: &'static str, v: &T) -> Self::Ok => {
        self.w.write_all(&*varint::encode(variant_index as i32))?;
        v.serialize(self)
    },
    serialize_tuple_struct(_name: &'static str, len: usize) -> Self::SerializeTupleStruct => Ok(self),
    serialize_tuple_variant(_name: &'static str, variant_index: u32, _variant: &'static str, len: usize) -> Self::SerializeTupleVariant => {
        self.w.write_all(&*varint::encode(variant_index as i32))?;
        Ok(self)
    },
    serialize_struct_variant(_name: &'static str, variant_index: u32, _variant: &'static str, len: usize) -> Self::SerializeStructVariant => {
        self.w.write_all(&*varint::encode(variant_index as i32))?;
        Ok(self)
    },
}

#[derive(Debug,PartialEq)]
enum Mode {
    VarInt,
    Rest,
    IntPrefixed,
    ShortPrefixed
}

struct SpecialSerializer<'de, W: Write> { w: &'de mut Serializer<W>, mode: Mode }
build_ser! {
    ['de, W: Write] &'de mut SpecialSerializer<'de, W>, &'de mut Serializer<W> => self, v:
    serialize_bool: bool => Err(Error::Any),
    serialize_i8: i8 =>   self.w.w.write_all(&*varint::encode(v as i32)),
    serialize_u8: u8 =>   self.w.w.write_all(&*varint::encode(v as i32)),
    serialize_i16: i16 => self.w.w.write_all(&*varint::encode(v as i32)),
    serialize_u16: u16 => self.w.w.write_all(&*varint::encode(v as i32)),
    serialize_i32: i32 => self.w.w.write_all(&*varint::encode(v as i32)),
    serialize_u32: u32 => self.w.w.write_all(&*varint::encode(v as i32)),
    serialize_i64: i64 => self.w.w.write_all(&*varint::encode(v as i32)),
    serialize_u64: u64 => self.w.w.write_all(&*varint::encode(v as i32)),
    serialize_f32: f32 => self.w.w.write_all(&*varint::encode(v as i32)),
    serialize_f64: f64 => self.w.w.write_all(&*varint::encode(v as i32)),
    serialize_char: char => Err(Error::Type("char")),
    serialize_str: &str => Err(Error::Type("str")),
    serialize_bytes: &[u8] => self.w.w.write_all(v),
    serialize_unit_struct: &'static str => Err(Error::Type("unit_struct")),

    serialize_some[T: ?Sized + Serialize](v: &T) -> Self::Ok => Err(Error::Type("uh")),
    serialize_none() -> Self::Ok => Err(Error::Type("uh")),
    serialize_newtype_struct[T: ?Sized + Serialize](name: &'static str, v: &T) -> Self::Ok => Err(Error::Type("uh")),
    serialize_seq(len: Option<usize>) -> Self::SerializeSeq => {
        match self.mode {
            Mode::IntPrefixed => {
                self.w.serialize_i32(len.ok_or(Error::Type("no length"))? as i32)?;
                Ok(&mut self.w)
            }
            Mode::ShortPrefixed => {
                self.w.serialize_i16(len.ok_or(Error::Type("eh"))? as i16)?;
                Ok(&mut self.w)
            }
            Mode::Rest => {
                Ok(&mut self.w)
            }
            ref c => {
                Err(Error::Custom(format!("you've done something weird: {:?}", c)))
            }
        }
    },
    serialize_tuple(len: usize) -> Self::SerializeTuple => Err(Error::Type("uh")),
    serialize_map(len: Option<usize>) -> Self::SerializeMap => Err(Error::Type("uh")),
    serialize_struct(_name: &'static str, _len: usize) -> Self::SerializeStruct => Err(Error::Type("uh")),
    serialize_unit() -> Self::Ok => Err(Error::Type("uh")),
    serialize_unit_variant(name: &'static str, variant_index: u32, _variant: &'static str) -> Self::Ok => Err(Error::Type("uh")),
    serialize_newtype_variant[T: ?Sized+Serialize](name: &'static str, variant_index: u32, _variant: &'static str, v: &T) -> Self::Ok => Err(Error::Type("uh")),
    serialize_tuple_struct(_name: &'static str, len: usize) -> Self::SerializeTupleStruct => Err(Error::Type("uh")),
    serialize_tuple_variant(_name: &'static str, variant_index: u32, _variant: &'static str, len: usize) -> Self::SerializeTupleVariant => Err(Error::Type("uh")),
    serialize_struct_variant(_name: &'static str, variant_index: u32, _variant: &'static str, len: usize) -> Self::SerializeStructVariant => Err(Error::Type("uh")),
}

impl<'a, W: Write> ser::SerializeSeq for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T: ?Sized+Serialize>(&mut self, value: &T) -> Result<(),Error> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<(),Error> { Ok(()) }
}

impl<'a, W: Write> ser::SerializeTuple for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;
    fn serialize_element<T: ?Sized+Serialize>(&mut self, value: &T) -> Result<(),Error> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<(),Error> { Ok(()) }
}
impl<'a, W: Write> ser::SerializeTupleStruct for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized+Serialize>(&mut self, value: &T) -> Result<(),Error> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<(),Error> { Ok(()) }
}
impl<'a, W: Write> ser::SerializeTupleVariant for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized+Serialize>(&mut self, value: &T) -> Result<(),Error> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<(),Error> { Ok(()) }
}
impl<'a, W: Write> ser::SerializeMap for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;
    fn serialize_key<T: ?Sized+Serialize>(&mut self, _value: &T) -> Result<(),Error> {
        Err(Error::Any)
    }
    fn serialize_value<T: ?Sized+Serialize>(&mut self, _value: &T) -> Result<(),Error> {
        Err(Error::Any)
    }
    fn end(self) -> Result<(),Error> { Ok(()) }
}
impl<'a, W: Write> ser::SerializeStruct for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized+Serialize>(&mut self, _key: &'static str, value: &T) -> Result<(),Error> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<(),Error> { Ok(()) }
}
impl<'a, W: Write> ser::SerializeStructVariant for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: ?Sized+Serialize>(&mut self, _key: &'static str, value: &T) -> Result<(),Error> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<(),Error> { Ok(()) }
}
