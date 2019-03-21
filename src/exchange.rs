use crate::error::Error as CoapError;
use crate::message::code::Method;
use crate::message::header::MessageType;
use crate::message::Message;
use crate::request::Request;
use crate::response::Response;
use std::net::SocketAddr;
use tokio::prelude::*;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot::{channel, Receiver, Sender as OneShotSender};

#[derive(Debug, Clone)]
pub struct ToSend(Message, SocketAddr);

#[derive(Debug, Clone, Copy)]
pub struct Key(SocketAddr, u16);

enum State {
    Handling(Receiver<Response>),
    Responding(Response),
}

pub struct Exchange {
    source: SocketAddr,
    method: Method,
    mid: u16,
    mtype: MessageType,
    tx: Sender<ToSend>,
    handle: Option<OneShotSender<Response>>,
    state: State,
}

impl Exchange {
    pub fn new(request: &Request, tx: Sender<ToSend>) -> Self {
        let (otx, orx) = channel();
        Self {
            source: *request.source(),
            method: *request.method(),
            mid: request.message_id(),
            mtype: request.message_type().clone(),
            tx,
            handle: Some(otx),
            state: State::Handling(orx),
        }
    }

    pub fn take_handle(&mut self) -> OneShotSender<Response> {
        self.handle
            .take()
            .expect("tried to take exchange handle more than once")
    }
}

impl Future for Exchange {
    type Item = (Key, Response);
    type Error = CoapError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(Async::NotReady)
    }
}
