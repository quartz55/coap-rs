use super::error::{FormatError, Result};
use byteorder::{ByteOrder, WriteBytesExt, BE};
use std::borrow::Cow;
use std::collections::BTreeMap;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Opts(BTreeMap<u16, Vec<Vec<u8>>>);

impl Opts {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn push<T: Opt>(&mut self, option: T) {
        self.0
            .entry(<T as Opt>::NUMBER)
            .or_insert_with(|| Vec::new())
            .push(option.to_bytes().into_owned())
    }

    pub fn push_raw(&mut self, num: u16, raw: &[u8]) {
        self.0
            .entry(num)
            .or_insert_with(|| Vec::new())
            .push(raw.to_vec())
    }

    pub fn get<T: Opt>(&self) -> Option<Vec<T>> {
        self.0.get(&<T as Opt>::NUMBER).map(|o| {
            o.iter()
                .map(|v| T::from_bytes(v.as_ref()).unwrap())
                .collect()
        })
    }
}

pub enum Class {
    Critical,
    Elective,
}

pub enum Proxy {
    SafeToForward,
    Unsafe,
}

pub enum Cache {
    NoCacheKey,
    CacheKey(u8),
}

pub trait Opt: Sized {
    const NUMBER: u16;
    type Format;

    fn new(value: Self::Format) -> Self;
    fn from_bytes(bytes: &[u8]) -> Result<Self>;
    fn to_bytes(&self) -> Cow<[u8]>;
    fn len(&self) -> usize;

    #[inline(always)]
    fn class() -> Class {
        match (Self::NUMBER & 0x0F) & 0b0001 {
            0 => Class::Elective,
            1 => Class::Critical,
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    fn proxy() -> Proxy {
        match (Self::NUMBER & 0x0F) & 0b0010 {
            0 => Proxy::SafeToForward,
            1 => Proxy::Unsafe,
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    fn cache_key() -> Cache {
        match (Self::NUMBER & 0x0F) >> 2 {
            0b0111 => Cache::NoCacheKey,
            c => Cache::CacheKey(c as u8),
        }
    }
}

macro_rules! option {
    (@make $name:ident no.($num:expr) length [$min:expr, $max:expr];
    $value:ident: $format:ty => [$to_bytes:expr, $len:expr];
    $bytes:ident => $from_bytes:expr) => {
        #[derive(PartialEq, Eq, Debug)]
        pub struct $name($format);
        impl Opt for $name {
            const NUMBER: u16 = $num;
            type Format = $format;

            fn new(value: $format) -> Self {
                Self(value)
            }

            fn from_bytes($bytes: &[u8]) -> Result<Self> {
                let len = $bytes.len();
                if len >= $min as usize && len <= $max as usize {
                    Ok(Self($from_bytes))
                } else {
                    Err(FormatError::InvalidOptionValue {
                        range: ($min..$max),
                        actual: len,
                    })?
                }
            }

            fn to_bytes(&self) -> Cow<[u8]> {
                let $value = &self.0;
                $to_bytes
            }

            fn len(&self) -> usize {
                let $value = &self.0;
                $len
            }
        }
        impl From<$format> for $name {
            fn from(value: $format) -> Self {
                Self::new(value)
            }
        }
    };
    (no.($num:expr) | $name:ident | empty) => {
        option! {
            @make $name no.($num) length [0, 0];
            _val: () => [{Cow::Borrowed(&[])}, 0];
            _bytes => {()}
        }
    };
    (no.($num:expr) | $name:ident | opaque[$min:expr, $max:expr]) => {
        option! {
            @make $name no.($num) length [$min, $max];
            val: Vec<u8> => [{Cow::Owned(val.clone())}, val.len()];
            bytes => {bytes.to_vec()}
        }
    };
    (no.($num:expr) | $name:ident | string[$min:expr, $max:expr]) => {
        option! {
            @make $name no.($num) length [$min, $max];
            val: String => [{Cow::Owned(val.clone().into_bytes())}, val.bytes().len()];
            bytes => {
                String::from_utf8(bytes.to_vec())
                    .or_else(|e| Err(FormatError::InvalidOption(e.to_string())))?
            }
        }
    };
    (no.($num:expr) | $name:ident | uint[$min:expr, $max:expr]) => {
        option! {
            @make $name no.($num) length [$min, $max];
            val: u64 => [{
                    let mut buf = vec![0, $max];
                    buf.write_u64::<BE>(*val).unwrap();
                    Cow::Owned(buf)
                }, {
                    let mut buf = vec![0, $max];
                    buf.write_u64::<BE>(*val).unwrap();
                    buf.len()
                }];
            bytes => {BE::read_u64(bytes)}
        }
    };
}

option!(no.(1)   | IfMatch       | opaque[0, 9]);
option!(no.(3)   | UriHost       | string[1, 8]);
option!(no.(4)   | ETag          | opaque[0, 8]);
option!(no.(5)   | IfNoneMatch   | empty);
option!(no.(6)   | Observe       | uint[0, 4]);
option!(no.(7)   | UriPort       | uint[0, 2]);
option!(no.(8)   | LocationPath  | string[0, 255]);
option!(no.(11)  | UriPath       | string[0, 255]);
option!(no.(12)  | ContentFormat | uint[0, 2]);
option!(no.(14)  | MaxAge        | uint[0, 4]);
option!(no.(15)  | UriQuery      | string[0, 255]);
option!(no.(17)  | Accept        | uint[0, 2]);
option!(no.(20)  | LocationQuery | string[0, 255]);
option!(no.(35)  | ProxyUri      | string[1, 1034]);
option!(no.(39)  | ProxyScheme   | string[1, 255]);
option!(no.(60)  | Size1         | uint[0, 4]);
option!(no.(284) | NoResponse    | uint[0, 4]);
