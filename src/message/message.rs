use super::code::Code;
use super::error::{Error, FormatError, Result};
use super::option::Opts;
use arrayvec::ArrayVec;
use byteorder::{ByteOrder, BE};
use std::iter::FromIterator;

const PAYLOAD_MARKER: u8 = 0xFF;
const HEADER_SIZE: usize = 4;

#[derive(Debug)]
pub struct Message {
    pub version: u8,
    pub mtype: MessageType,
    pub code: Code,
    pub mid: u16,
    pub token: ArrayVec<[u8; 8]>,
    pub options: Opts,
    pub payload: Option<Vec<u8>>,
}

impl Message {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < HEADER_SIZE {
            return Err(Error::PacketTooSmall(bytes.len()));
        }

        let (header, bytes) = bytes.split_at(HEADER_SIZE);

        // Parse header
        let version = header[0] >> 6;
        let mtype = (header[0] >> 4) & 0b11;
        let mtype = MessageType::from_u8(&mtype);
        let token_len = (header[0] & 0x0F) as usize;
        if token_len > 8 {
            return Err(FormatError::InvalidTokenLength(token_len))?;
        }
        if bytes.len() < token_len {
            return Err(FormatError::TokenLengthMismatch {
                actual: bytes.len(),
                expected: token_len,
            })?;
        }
        let code = Code::from_u8(header[1]);
        let mid = BE::read_u16(&header[2..]);

        // Parse token
        let (token, bytes) = bytes.split_at(token_len);
        let token = ArrayVec::from_iter(token.iter().cloned());

        // Parse options
        let mut i = 0;
        let mut options = Opts::new();
        let mut opt_num_offset = 0u16;
        while i < bytes.len() && bytes[i] != PAYLOAD_MARKER {
            let header = bytes[i];
            let (delta, offset) = match header >> 4 {
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
            let value = if val_i + length < bytes.len() {
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
            version,
            mtype,
            code,
            mid,
            token,
            options,
            payload,
        })
    }
}

#[derive(Debug)]
pub enum MessageType {
    Confirmable,
    NonConfirmable,
    Acknowledgement,
    Reset,
}

impl MessageType {
    pub fn from_u8(v: &u8) -> Self {
        match v & 0b11 {
            0 => MessageType::Confirmable,
            1 => MessageType::NonConfirmable,
            2 => MessageType::Acknowledgement,
            3 => MessageType::Reset,
            _ => unreachable!(),
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            MessageType::Confirmable => 0,
            MessageType::NonConfirmable => 1,
            MessageType::Acknowledgement => 2,
            MessageType::Reset => 3,
        }
    }
}
