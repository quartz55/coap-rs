use crate::message::code::SuccessCode;
use crate::message::{Body, Header, Message, MessageKind, MessageType, Opts, ResponseCode, Token};
use crate::request::Request;
use futures::future::Future;
use std::net::SocketAddr;

pub struct Response {
    dest: SocketAddr,
    code: ResponseCode,
    message_type: MessageType,
    token: Token,
    message_id: u16,
    options: Opts,
    payload: Option<Vec<u8>>,
}

impl Response {
    pub fn from_request(req: &Request) -> Self {
        Self {
            dest: req.source().clone(),
            code: ResponseCode::Success(SuccessCode::Content),
            message_type: MessageType::Acknowledgement,
            token: req.token().clone(),
            message_id: req.message_id(),
            options: Opts::new(),
            payload: None,
        }
    }

    pub fn serialize(&self) -> Message {
        let header = Header {
            version: 1,
            mtype: self.message_type.clone(),
            tkl: self.token.len(),
            code: self.code.as_raw_code(),
            message_id: self.message_id,
        };
        let body = Body {
            token: self.token.clone(),
            options: self.options.clone(),
            payload: self.payload.clone(),
        };
        Message {
            header,
            kind: MessageKind::Response(body),
        }
    }

    pub fn dest(&self) -> &SocketAddr {
        &self.dest
    }
    pub fn code(&self) -> &ResponseCode {
        &self.code
    }
    pub fn message_type(&self) -> &MessageType {
        &self.message_type
    }
    pub fn token(&self) -> &Token {
        &self.token
    }
    pub fn message_id(&self) -> u16 {
        self.message_id
    }
    pub fn options(&self) -> &Opts {
        &self.options
    }
    pub fn payload(&self) -> Option<&[u8]> {
        match self.payload {
            Some(ref pl) => Some(pl.as_slice()),
            None => None,
        }
    }
}

pub enum Carry {
    Piggyback(Response),
    // Seperate(Box<Future<Item = Response, Error = ()>>),
}
