use crate::message::code::SuccessCode;
use crate::message::{Message, MessageBuilder, MessageType, Opts, ResponseCode, Token};
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
        let m = MessageBuilder::response()
            .acknowledgement()
            .message_id(self.message_id)
            .response_code(self.code)
            .token(self.token.clone());
        let m = match self.payload {
            None => m,
            Some(ref pl) => m.payload(pl.clone()),
        };
        m.build()
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
