use crate::codec::ParsedMsg;
use crate::error::{self, Error as CoapError, ErrorKind, Result as CoapResult};
use crate::exchange::{Exchange, Key, ToSend};
use crate::message::{code::SuccessCode, Message, MessageBuilder, MessageKind};
use crate::midgen::MidGen;
use crate::reliability::Reliablity;
use crate::request::Request;
use crate::response::{Carry, Response};
use crate::socket::CoapSocket;
use futures::try_ready;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use tokio::prelude::*;
use tokio::sync::mpsc;

fn default_handler(req: Request, mut res: Response) -> impl Future<Item = Carry, Error = ()> {
    res.set_payload(String::from("shitface"));
    futures::future::ok(res.into())
    // futures::future::ok(Carry::Seperate(
    //     Box::new(futures::future::ok(res)),
    //     Reliablity::NonConfirmable,
    // ))
}

pub struct Server {
    socket: CoapSocket,
    rx: mpsc::Receiver<ToSend>,
    tx: mpsc::Sender<ToSend>,
    mid: MidGen,
    exchanges: HashMap<Key, Exchange>,
}

impl Server {
    pub fn new<A: ToSocketAddrs>(addr: A) -> CoapResult<Self> {
        let socket = CoapSocket::new(addr)?;
        let (tx, rx) = mpsc::channel(1024);
        return Ok(Self {
            socket,
            rx,
            tx,
            mid: MidGen::new(),
            exchanges: HashMap::new(),
        });
    }

    fn poll_exchanges(&mut self) -> Poll<(), CoapError> {
        let mut done = vec![];
        for (_, e) in self.exchanges.iter_mut() {
            let is_done = match e.poll(&mut self.mid) {
                Ok(Async::Ready(_)) => true,
                Ok(Async::NotReady) => false,
                Err(err) => {
                    match err.kind() {
                        ErrorKind::RequestCancelled => warn!("Request cancelled"),
                        ErrorKind::ResponseTimeout => warn!("Response timeout"),
                        _ => return Err(err),
                    };
                    true
                }
            };
            if is_done {
                done.push(e.key());
            }
        }
        done.iter().for_each(|k| {
            self.exchanges.remove(k);
        });
        Ok(Async::NotReady)
    }

    fn poll_outgoing(&mut self) -> Poll<(), CoapError> {
        loop {
            match self
                .rx
                .poll()
                .map_err(|_| CoapError::broken_channel("server rx"))?
            {
                Async::Ready(None) => panic!("boom"),
                Async::Ready(Some(ToSend(msg, addr))) => {
                    info!("Outgoing message\n{}", msg);
                    info!(
                        "[{}]",
                        msg.as_bytes()
                            .unwrap()
                            .iter()
                            .fold(String::new(), |acc, b| acc + &format!("{:#X}, ", b))
                    );
                    self.socket.send(msg, addr);
                    continue;
                }
                Async::NotReady => break,
            };
        }
        self.socket.poll()
    }

    fn poll_incoming(&mut self) -> Poll<(), CoapError> {
        loop {
            let (inc, src) = try_ready!(self.socket.poll_recv());
            let msg = match inc {
                ParsedMsg::Valid(msg) => msg,
                ParsedMsg::Reject(header, err) => {
                    warn!(
                        "Rejecting CoAP message with invalid format: {}",
                        error::pprint_error(&err)
                    );
                    let rst = MessageBuilder::reset(header.message_id()).build();
                    self.socket.send(rst, src);
                    continue;
                }
                ParsedMsg::Invalid(err) => {
                    warn!(
                        "Silently ignoring invalid CoAP message: {}",
                        error::pprint_error(&err)
                    );
                    continue;
                }
            };

            debug!("Incoming message\n{0:?}\n{0}", msg);

            if msg.is_reserved() {
                warn!("Silently ignoring message using reserved code:\n{}", msg);
                continue;
            }

            let key = Key::new(src, msg.header().message_id());
            match (msg.kind(), self.exchanges.get_mut(&key)) {
                (MessageKind::Request(_), None) => {
                    let req = Request::from_message(src, msg).unwrap();
                    let mut exch = Exchange::new(&req, self.tx.clone());
                    let handle = exch.take_handle();
                    let res = Response::from_request(&req);
                    tokio::spawn(default_handler(req, res).map(
                        move |res| match handle.send(res) {
                            _ => (),
                        },
                    ));
                    // Trigger wakeup (if needed)
                    task::current().notify();
                    self.exchanges.insert(exch.key(), exch);
                }
                (MessageKind::Request(_), Some(_)) => {
                    warn!("Ignoring request for existing exchange:\n{}", msg);
                }
                (_, Some(exch)) => exch.handle(msg),
                _ => {
                    warn!("Rejecting message due to lack of context:\n{}", msg);
                    let rst = MessageBuilder::reset(msg.header().message_id()).build();
                    self.tx
                        .try_send(ToSend(rst, src))
                        .map_err(|_| CoapError::broken_channel("server socket"))?;
                }
            }
        }
    }
}

impl Future for Server {
    type Item = ();
    type Error = CoapError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            self.poll_exchanges()?;

            self.poll_outgoing()?;

            try_ready!(self.poll_incoming());
        }
    }
}
