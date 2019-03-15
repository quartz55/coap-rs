use super::body::Body;
use super::code::{Method, RawCode, ResponseCode};
use super::error::{Error, FormatError, Result};
use super::header::Header;
use super::header::MessageType;
use crate::params::HEADER_SIZE;
use std::fmt;

#[derive(Debug, Clone)]
pub enum MessageKind {
    Empty,
    Request(Body),
    Response(Body),
    Reserved(Body),
}

#[derive(Debug, Clone)]
pub struct Message {
    pub header: Header,
    pub kind: MessageKind,
}

impl Message {
    pub fn empty(message_type: MessageType, message_id: u16) -> Self {
        Self {
            header: Header {
                version: 1,
                mtype: message_type,
                tkl: 0,
                code: RawCode(0, 00),
                message_id,
            },
            kind: MessageKind::Empty,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < HEADER_SIZE {
            return Err(Error::PacketTooSmall(bytes.len()));
        }

        // Parse header
        let (header, bytes) = bytes.split_at(HEADER_SIZE);
        let header = Header::from_bytes(header)?;

        // Handle Empty message special case (code 0.00)
        if header.code == RawCode(0, 00) {
            if header.tkl != 0 {
                return Err(FormatError::InvalidEmptyCode(
                    "token length MUST be 0".into(),
                ))?;
            }
            if bytes.len() != 0 {
                return Err(FormatError::InvalidEmptyCode(
                    "bytes MUST NOT be present after message ID".into(),
                ))?;
            }
            return Ok(Self {
                header,
                kind: MessageKind::Empty,
            });
        }

        let body = Body::from_bytes(&header, bytes)?;

        let kind = match header.code.class() {
            0 => MessageKind::Request(body),
            2..=5 => MessageKind::Response(body),
            _ => MessageKind::Reserved(body),
        };

        Ok(Self { header, kind })
    }

    pub fn as_bytes(&self) -> Result<Vec<u8>> {
        use MessageKind::*;
        match &self.kind {
            Empty => Ok(self.header.to_bytes()?.to_vec()),
            Request(body) | Response(body) | Reserved(body) => {
                let mut body = body.to_bytes();
                let mut buf = Vec::with_capacity(HEADER_SIZE + body.len());
                buf.append(&mut self.header.to_bytes()?.to_vec());
                buf.append(&mut body);
                Ok(buf)
            }
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            MessageKind::Empty => {
                write!(f, "-------- EMPTY --------\n")?;
                write!(f, "Header: {}", self.header)
            }
            kind => {
                let body = match kind {
                    MessageKind::Request(body)
                    | MessageKind::Response(body)
                    | MessageKind::Reserved(body) => body,
                    _ => unreachable!(),
                };
                let (kind, code) = match kind {
                    MessageKind::Request(_) => (
                        "REQUEST",
                        Method::from_raw_code(self.header.code).unwrap().to_string(),
                    ),
                    MessageKind::Response(_) => (
                        "RESPONSE",
                        ResponseCode::from_raw_code(self.header.code)
                            .unwrap()
                            .to_string(),
                    ),
                    MessageKind::Reserved(_) => ("RESERVED", self.header.code.to_string()),
                    _ => unreachable!(),
                };
                write!(f, "-------- {} --------\n", kind)?;
                write!(f, "Header: {} {}\n", code, self.header)?;
                write!(f, "Token: {}", body.token)?;
                if let Some(ref pl) = body.payload {
                    write!(f, "\nPayload: {} bytes", pl.len())?;
                }
                Ok(())
            }
        }
    }
}
