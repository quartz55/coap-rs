use crate::message::{Body, Header, Message, MessageKind, MessageType, Method, Opts, Token};
use std::net::SocketAddr;

#[derive(Debug)]
pub struct Request {
    source: SocketAddr,
    method: Method,
    message_id: u16,
    message_type: MessageType,
    body: Body,
}

impl Request {
    pub fn from_message(source: SocketAddr, message: Message) -> Option<Self> {
        let Header {
            code,
            message_id,
            mtype,
            ..
        } = message.header;
        match message.kind {
            MessageKind::Request(body) => Some(Self {
                source,
                method: Method::from_raw_code(code).unwrap(),
                message_id,
                message_type: mtype,
                body,
            }),
            _ => None,
        }
    }
    pub fn source(&self) -> &SocketAddr {
        &self.source
    }
    pub fn message_id(&self) -> u16 {
        self.message_id
    }
    pub fn message_type(&self) -> &MessageType {
        &self.message_type
    }
    pub fn method(&self) -> &Method {
        &self.method
    }
    pub fn token(&self) -> &Token {
        &self.body.token
    }
    pub fn options(&self) -> &Opts {
        &self.body.options
    }
    pub fn payload(&self) -> Option<&[u8]> {
        match self.body.payload {
            None => None,
            Some(ref pl) => Some(pl.as_slice()),
        }
    }
}
