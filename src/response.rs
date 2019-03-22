use crate::message::builder::{No, ResponseBuilder, Yes};
use crate::message::code::SuccessCode;
use crate::message::{MessageBuilder, Opts, ResponseCode, Token};
use crate::reliability::Reliablity;
use crate::request::Request;
use std::net::SocketAddr;
use tokio::prelude::*;

#[derive(Debug, Clone)]
pub struct Response {
    dest: SocketAddr,
    code: ResponseCode,
    options: Opts,
    payload: Option<Vec<u8>>,
}

impl Response {
    pub(crate) fn from_request(req: &Request) -> Self {
        Self {
            dest: req.source().clone(),
            code: ResponseCode::Success(SuccessCode::Content),
            options: Opts::new(),
            payload: None,
        }
    }

    pub fn dest(&self) -> &SocketAddr {
        &self.dest
    }
    pub fn code(&self) -> &ResponseCode {
        &self.code
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
    pub fn set_payload<P>(&mut self, payload: P)
    where
        P: Into<Vec<u8>>,
    {
        self.payload = Some(payload.into());
    }

    pub(crate) fn serialize(&self) -> ResponseBuilder<No, No> {
        let m = MessageBuilder::response().response_code(self.code);
        let m = match self.payload {
            None => m,
            Some(ref pl) => m.payload(pl.clone()),
        };
        m
    }
}

pub type Seperate = Box<dyn Future<Item = Response, Error = ()> + Send>;

pub enum Carry {
    Piggyback(Response),
    Seperate(Seperate, Reliablity),
}

impl From<Response> for Carry {
    fn from(r: Response) -> Carry {
        Carry::Piggyback(r)
    }
}

impl<F> From<F> for Carry
where
    F: Future<Item = Response, Error = ()> + Send + 'static,
{
    fn from(f: F) -> Carry {
        Carry::Seperate(Box::new(f), Reliablity::Confirmable)
    }
}
