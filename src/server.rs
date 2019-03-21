use crate::codec::Incoming;
use crate::error::{self, Error as CoapError, Result as CoapResult};
use crate::exchange::{Exchange, ToSend};
use crate::message::{code::SuccessCode, Message, MessageBuilder, MessageKind};
use crate::request::Request;
use crate::socket::CoapSocket;
use futures::try_ready;
use log::{debug, error, info, warn};
use std::collections::{HashMap, VecDeque};
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use tokio::prelude::*;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
struct MidGen(HashMap<IpAddr, u16>);
impl Default for MidGen {
    fn default() -> Self {
        Self(HashMap::new())
    }
}
impl MidGen {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn next(&mut self, source: SocketAddr) -> u16 {
        *self
            .0
            .entry(source.ip())
            .and_modify(|e| *e += 1)
            .or_insert(0)
    }
}

pub struct Server {
    socket: CoapSocket,
    rx: mpsc::Receiver<ToSend>,
    tx: mpsc::Sender<ToSend>,
    mid: MidGen,
    exchangres: VecDeque<Exchange>
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
        });
    }
}

impl Future for Server {
    type Item = ();
    type Error = CoapError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            self.socket.poll().unwrap();

            let (inc, src) = try_ready!(self.socket.poll_recv());
            let msg = match inc {
                Incoming::Valid(msg) => msg,
                Incoming::Reject(header, err) => {
                    warn!(
                        "Rejecting CoAP message with invalid format: {}",
                        error::pprint_error(&err)
                    );
                    let rst = MessageBuilder::reset(header.message_id).build();
                    self.socket.send(rst, src);
                    continue;
                }
                Incoming::Invalid(err) => {
                    warn!(
                        "Silently ignoring invalid CoAP message: {}",
                        error::pprint_error(&err)
                    );
                    continue;
                }
            };

            println!("{:?}", msg);
            println!("{}", msg);

            match msg.kind {
                MessageKind::Request(_) => {
                    let req = Request::from_message(src, msg).unwrap();
                    let exch = 
                }
                MessageKind::Empty => {}
                MessageKind::Response(_) => {
                    error!("response what?\n{}", msg);
                }
                MessageKind::Reserved(_) => {
                    warn!("Silently ignoring message using reserved code:\n{}", msg);
                }
            }
        }
    }
}
