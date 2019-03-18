use crate::codec::{CoapCodec, Msg};
use crate::error::{self, Error as CoapError, Result as CoapResult};
use crate::message::{code::SuccessCode, Message, MessageBuilder, MessageKind};
use crate::request::Request;
use futures::sink::SendAll;
use futures::stream::{iter_ok, IterOk};
use futures::try_ready;
use log::{debug, error, info, warn};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::io;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::vec::Drain;
use tokio::net::{UdpFramed, UdpSocket};
use tokio::prelude::stream::{SplitSink, SplitStream};
use tokio::prelude::*;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
struct MidGen(u16);
// struct MidGen(HashMap<SocketAddr, u16>);
impl Default for MidGen {
    fn default() -> Self {
        // Self(HashMap::new())
        Self(0)
    }
}
impl MidGen {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn next(&mut self, source: SocketAddr) -> u16 {
        // *self.0.entry(source).and_modify(|e| *e += 1).or_insert(0)
        let n = self.0;
        self.0 += 1;
        n
    }
}

type Sock = UdpFramed<CoapCodec>;

#[derive(Debug, Clone)]
struct ToSend(Message, SocketAddr);
struct Socket {
    outgoing: Vec<(Message, SocketAddr)>,
    sock_in: SplitStream<Sock>,
    sock_out: Option<SplitSink<Sock>>,
    sending: Option<
        SendAll<SplitSink<Sock>, IterOk<std::vec::IntoIter<(Message, SocketAddr)>, CoapError>>,
    >,
}
impl Socket {
    pub fn new(socket: UdpSocket) -> Self {
        let (sock_out, sock_in) = UdpFramed::new(socket, CoapCodec).split();
        Self {
            outgoing: Vec::new(),
            sock_in,
            sock_out: Some(sock_out),
            sending: None,
        }
    }

    pub fn send(&mut self, msg: Message, dest: SocketAddr) {
        self.outgoing.push((msg, dest));
    }

    pub fn poll(&mut self) -> Poll<(), CoapError> {
        loop {
            if let Some(mut snd) = self.sending.take() {
                let sock = match snd.poll()? {
                    Async::Ready((sock, _)) => sock,
                    Async::NotReady => {
                        self.sending = Some(snd);
                        return Ok(Async::NotReady);
                    }
                };
                self.sock_out = Some(sock);
            }
            if self.outgoing.len() > 0 {
                let out = self.outgoing.drain(..).collect::<Vec<_>>();
                println!("sending {:?}", out);
                self.sending = Some(self.sock_out.take().unwrap().send_all(iter_ok(out)));
            } else {
                return Ok(Async::NotReady);
            }
        }
    }

    pub fn poll_recv(&mut self) -> Poll<(Msg, SocketAddr), CoapError> {
        match self.sock_in.poll()? {
            Async::Ready(Some(inc)) => Ok(Async::Ready(inc)),
            Async::Ready(None) => panic!("what"),
            Async::NotReady => Ok(Async::NotReady),
        }
    }
}

pub struct Server {
    socket: Socket,
    rx: mpsc::Receiver<ToSend>,
    tx: mpsc::Sender<ToSend>,
    mid: MidGen,
}

impl Server {
    pub fn new<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        for addr in addr.to_socket_addrs()? {
            let socket = match UdpSocket::bind(&addr) {
                Ok(socket) => socket,
                Err(_) => continue,
            };
            let socket = Socket::new(socket);
            let (tx, rx) = mpsc::channel(1024);
            return Ok(Self {
                socket,
                rx,
                tx,
                mid: MidGen::new(),
            });
        }
        return Err(io::ErrorKind::AddrNotAvailable.into());
    }
}

impl Future for Server {
    type Item = ();
    type Error = CoapError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            self.socket.poll().unwrap();

            let (msg, src) = try_ready!(self.socket.poll_recv());
            let msg = match msg {
                Msg::Valid(msg) => msg,
                Msg::Invalid(header, err) => {
                    warn!(
                        "Received CoAP message with invalid format: {}",
                        error::pprint_error(&err)
                    );
                    // warn!("!UNIMPLEMENTED! Should send matching Reset message");
                    let rst = MessageBuilder::reset(header.message_id).build();
                    self.socket.send(rst, src);
                    continue;
                }
            };

            println!("{:?}", msg);
            println!("{}", msg);

            match msg.kind {
                MessageKind::Request(_) => {
                    let req = Request::from_message(src, msg).unwrap();
                    let ack = MessageBuilder::empty()
                        .acknowledgement()
                        .message_id(req.message_id())
                        .build();
                    println!("ack:\n{}", ack);
                    self.socket.send(ack, *req.source());
                    let res = MessageBuilder::response()
                        .confirmable()
                        .message_id(self.mid.next(src))
                        .token(req.token().clone())
                        .response_code(SuccessCode::Valid)
                        .build();
                    println!("res:\n{}", res);
                    self.socket.send(res, *req.source());
                }
                MessageKind::Empty => {}
                MessageKind::Response(_) => {
                    warn!("what?");
                }
                MessageKind::Reserved(_) => {
                    warn!("Ignoring message using reserved codes");
                }
            }
        }
    }
}
