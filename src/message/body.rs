use super::error::{FormatError, Result};
use super::header::Header;
use super::option::Opts;
use super::token::Token;
use crate::params::PAYLOAD_MARKER;
use byteorder::{ByteOrder, BE};

#[derive(Debug, Clone)]
pub struct Body {
    pub token: Token,
    pub options: Opts,
    pub payload: Option<Vec<u8>>,
}

impl Body {
    pub fn new(token: Token, options: Opts, payload: Option<Vec<u8>>) -> Self {
        Self {
            token,
            options,
            payload,
        }
    }

    pub fn from_bytes(header: &Header, bytes: &[u8]) -> Result<Self> {
        // Parse token
        if bytes.len() < header.tkl {
            return Err(FormatError::TokenLengthMismatch {
                actual: bytes.len(),
                expected: header.tkl,
            })?;
        }
        let (token, bytes) = bytes.split_at(header.tkl);
        let token = Token::new(token);

        // Parse options
        // https://tools.ietf.org/html/rfc7252#section-3.1
        let mut i = 0;
        let mut options = Opts::new();
        let mut opt_num_offset = 0u16;
        while i < bytes.len() && bytes[i] != PAYLOAD_MARKER {
            let header = bytes[i];
            let (delta, offset) = match (header & 0xF0) >> 4 {
                d if d <= 12 => (d as u16, 1),
                13 if i + 1 < bytes.len() => (bytes[i + 1] as u16 + 13, 2),
                14 if i + 2 < bytes.len() => (BE::read_u16(&bytes[i + 1..i + 3]) + 269, 3),
                _ => return Err(FormatError::InvalidOptionDelta)?,
            };
            let (length, offset) = match header & 0x0F {
                d if d <= 12 => (d as u16, offset),
                13 if i + offset + 1 < bytes.len() => (bytes[i + offset] as u16 + 13, offset + 1),
                14 if i + offset + 2 < bytes.len() => (
                    BE::read_u16(&bytes[(i + offset)..(i + offset + 2)]) + 269,
                    offset + 2,
                ),
                _ => return Err(FormatError::InvalidOptionLength)?,
            };

            let opt_num = opt_num_offset + delta;
            opt_num_offset = opt_num;

            let length = length as usize;
            let val_i = i + offset;
            let value = if val_i + length <= bytes.len() {
                &bytes[val_i..val_i + length]
            } else {
                return Err(FormatError::OptionLengthMismatch {
                    actual: bytes.len() - val_i,
                    expected: length,
                })?;
            };

            options.push_raw(opt_num, value);

            i += offset + length;
        }

        // Parse payload
        let payload = match i < bytes.len() {
            false => None,
            true => {
                let rest = &bytes[i..];
                match (rest[0], rest.len()) {
                    (PAYLOAD_MARKER, len) if len <= 1 => {
                        return Err(FormatError::UnexpectedPayloadMarker)?;
                    }
                    (PAYLOAD_MARKER, _) => Some(rest[1..].to_vec()),
                    _ => None,
                }
            }
        };

        Ok(Self {
            token,
            options,
            payload,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let size = self.token.len() + self.payload.as_ref().map_or(0, |pl| pl.len() + 1);
        let mut buf = Vec::with_capacity(size);
        buf.extend(self.token.to_bytes().iter());
        if let Some(ref pl) = self.payload {
            buf.push(PAYLOAD_MARKER);
            buf.extend(pl.iter());
        }
        buf
    }
}
