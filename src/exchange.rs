use crate::error::Error as CoapError;
use crate::message::code::Method;
use crate::message::token::Token;
use crate::message::{Message, MessageBuilder};
use crate::midgen::MidGen;
use crate::params::{ACK_RANDOM_FACTOR, ACK_TIMEOUT, MAX_RETRANSMIT};
use crate::reliability::Reliablity;
use crate::request::Request;
use crate::response::{Carry, Response, Seperate};
use futures::try_ready;
use log::{debug, info};
use rand::Rng;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::prelude::*;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot::{channel, Receiver as OsReceiver, Sender as OsSender};
use tokio::timer::Delay;

#[derive(Debug, Clone)]
pub struct ToSend(pub Message, pub SocketAddr);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Key(pub SocketAddr, pub u16);
impl Key {
    #[inline]
    pub fn new(addr: SocketAddr, mid: u16) -> Self {
        Self(addr, mid)
    }

    pub fn from_request(request: &Request) -> Self {
        Self(request.source(), request.message_id())
    }
}

#[derive(Debug)]
struct Transmition {
    delay: Delay,
    retr: u32,
}

impl Transmition {
    pub fn new() -> Self {
        let init_delay = rand::thread_rng()
            .gen_range(ACK_TIMEOUT, (ACK_TIMEOUT as f64 * ACK_RANDOM_FACTOR) as u64);
        let delay = Delay::new(Instant::now() + Duration::from_millis(init_delay));
        Self { delay, retr: 0 }
    }

    pub fn poll_retry(&mut self) -> Poll<(), ()> {
        loop {
            match self.delay.poll().unwrap() {
                Async::Ready(_) if self.retr < MAX_RETRANSMIT => {
                    self.retr += 1;
                    let timeout =
                        Instant::now() + Duration::from_millis(ACK_TIMEOUT * 2u64.pow(self.retr));
                    self.delay.reset(timeout);
                    // Trigger wakeup
                    self.delay.poll().unwrap();
                    return Ok(Async::Ready(()));
                }
                Async::Ready(_) => return Err(()),
                Async::NotReady => return Ok(Async::NotReady),
            }
        }
    }
}

enum State {
    Handling(OsReceiver<Carry>),
    HandlingSep(Seperate, Reliablity),
    Responding(Transmition, Response, Message),
    Done(Response),
    Cancelled,
}

pub struct Exchange {
    source: SocketAddr,
    method: Method,
    mid: u16,
    reliablity: Reliablity,
    token: Token,
    tx: Sender<ToSend>,
    handle: Option<OsSender<Carry>>,
    state: State,
}

impl Exchange {
    pub fn new(request: &Request, tx: Sender<ToSend>) -> Self {
        let (otx, orx) = channel();
        Self {
            source: request.source(),
            method: request.method(),
            mid: request.message_id(),
            reliablity: request.reliablity(),
            token: request.token().clone(),
            tx,
            handle: Some(otx),
            state: State::Handling(orx),
        }
    }

    pub fn take_handle(&mut self) -> OsSender<Carry> {
        self.handle
            .take()
            .expect("tried to take exchange handle more than once")
    }

    pub fn handle(&mut self, message: Message) {
        if message.is_reset() {
            self.state = State::Cancelled;
        }
    }

    #[inline]
    pub fn key(&self) -> Key {
        Key(self.source, self.mid)
    }

    fn try_send(&mut self, message: Message) -> Result<(), CoapError> {
        self.tx
            .try_send(ToSend(message, self.source))
            .map_err(|_| CoapError::broken_channel("server tx"))
    }

    pub fn poll(&mut self, midgen: &mut MidGen) -> Poll<(), CoapError> {
        loop {
            match self.state {
                State::Handling(ref mut rx) => {
                    let res = try_ready!(rx
                        .poll()
                        .map_err(|_| CoapError::broken_channel("exchange rx")));
                    match res {
                        Carry::Piggyback(res) => {
                            // Send response back as Acknowledgement
                            let res_ack = res
                                .serialize()
                                .acknowledgement()
                                .message_id(self.mid)
                                .token(self.token.clone())
                                .build();
                            self.try_send(res_ack)?;
                            self.state = State::Done(res);
                        }
                        Carry::Seperate(sep, rel) => {
                            match self.reliablity {
                                Reliablity::Confirmable => {
                                    let ack = MessageBuilder::empty()
                                        .acknowledgement()
                                        .message_id(self.mid)
                                        .build();
                                    self.try_send(ack)?;
                                }
                                Reliablity::NonConfirmable => {
                                    debug!("Not sending empty ACK because non confirmable")
                                }
                            };
                            self.state = State::HandlingSep(sep, rel);
                        }
                    };
                }
                State::HandlingSep(ref mut sep, ref rel) => {
                    let res = try_ready!(sep.poll().map_err(|_| CoapError::handler()));
                    match rel {
                        Reliablity::Confirmable => {
                            let con_msg = res
                                .serialize()
                                .message_id(midgen.next(self.source))
                                .token(self.token.clone())
                                .confirmable()
                                .build();
                            self.try_send(con_msg.clone())?;
                            self.state = State::Responding(Transmition::new(), res, con_msg);
                        }
                        Reliablity::NonConfirmable => {
                            let non_res = res
                                .serialize()
                                .non_confirmable()
                                .message_id(midgen.next(self.source))
                                .token(self.token.clone())
                                .build();
                            self.try_send(non_res)?;
                            self.state = State::Done(res);
                        }
                    }
                }
                State::Responding(ref mut trans, ref res, ref con_msg) => {
                    try_ready!(trans
                        .poll_retry()
                        .map_err(|_| CoapError::response_timeout()));
                    info!("retry X{}\n{:?}", trans.retr, res);
                    self.try_send(con_msg.clone())?;
                }
                State::Done(ref res) => return Ok(Async::Ready(())),
                State::Cancelled => return Err(CoapError::request_cancelled()),
            }
        }
    }
}
