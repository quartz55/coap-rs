use crate::message::{Header, Message, MessageKind, Method, Opts, Token};
use crate::reliability::Reliablity;
use std::net::SocketAddr;

#[derive(Debug)]
pub struct Request {
    source: SocketAddr,
    method: Method,
    message_id: u16,
    reliablity: Reliablity,
    token: Token,
    options: Opts,
    payload: Option<Vec<u8>>,
}

impl Request {
    pub fn from_message(source: SocketAddr, message: Message) -> Option<Self> {
        let (header, kind) = message.consume();
        match kind {
            MessageKind::Request(body) => Some(Self {
                source,
                method: Method::from_raw_code(header.code()).unwrap(),
                message_id: header.message_id(),
                reliablity: Reliablity::from_message_type(header.message_type())?,
                token: body.token,
                options: body.options,
                payload: body.payload,
            }),
            _ => None,
        }
    }
    pub fn source(&self) -> SocketAddr {
        self.source
    }
    pub fn message_id(&self) -> u16 {
        self.message_id
    }
    pub fn reliablity(&self) -> Reliablity {
        self.reliablity
    }
    pub fn method(&self) -> Method {
        self.method
    }
    pub fn token(&self) -> &Token {
        &self.token
    }
    pub fn options(&self) -> &Opts {
        &self.options
    }
    pub fn payload(&self) -> Option<&[u8]> {
        match self.payload {
            None => None,
            Some(ref pl) => Some(&pl),
        }
    }
}
