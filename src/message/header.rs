use super::error::{FormatError, Result};
use crate::message::code::RawCode;
use crate::params::VERSION;
use byteorder::{ByteOrder, BE};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Header {
    pub version: u8,
    pub mtype: MessageType,
    pub tkl: usize,
    pub code: RawCode,
    pub message_id: u16,
}

impl Header {
    pub fn new(message_type: MessageType, tkl: usize, code: RawCode, message_id: u16) -> Self {
        Self {
            version: VERSION,
            mtype: message_type,
            tkl,
            code,
            message_id,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let version = bytes[0] >> 6;
        if version != VERSION {
            return Err(FormatError::UnknownVersion(version))?;
        }
        let mtype = (bytes[0] >> 4) & 0b11;
        let mtype = MessageType::from_u8(&mtype);
        let tkl = (bytes[0] & 0x0F) as usize;
        if tkl > 8 {
            return Err(FormatError::InvalidTokenLength(tkl))?;
        }
        let code = RawCode::from_u8(bytes[1]);
        let mid = BE::read_u16(&bytes[2..]);
        Ok(Self {
            version,
            mtype,
            tkl,
            code,
            message_id: mid,
        })
    }

    pub fn to_bytes(&self) -> Result<[u8; 4]> {
        if self.tkl > 8 {
            return Err(FormatError::InvalidTokenLength(self.tkl))?;
        }
        let mut buf = [0u8, 0, 0, 0];
        buf[0] = (self.version << 6) | (self.mtype.as_u8() << 4) | (self.tkl as u8);
        buf[1] = self.code.as_u8();
        BE::write_u16(&mut buf[2..], self.message_id);
        Ok(buf)
    }
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "(T={}, Code={}, MID={:#x})",
            self.mtype, self.code, self.message_id
        )
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
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
impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MessageType::Confirmable => write!(f, "CON"),
            MessageType::NonConfirmable => write!(f, "NON"),
            MessageType::Acknowledgement => write!(f, "ACK"),
            MessageType::Reset => write!(f, "RST"),
        }
    }
}
