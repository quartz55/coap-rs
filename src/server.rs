use crate::codec::{CoapCodec, Msg};
use crate::error::{self, Error as CoapError, Result as CoapResult};
use crate::message::{Message, MessageBuilder, MessageKind};
use crate::request::Request;
use log::{debug, error, info, warn};
use std::io;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use tokio::net::{UdpFramed, UdpSocket};
use tokio::prelude::*;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
struct MidGen(u64);
impl Default for MidGen {
    fn default() -> Self {
        Self(0)
    }
}
impl MidGen {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn next(&mut self) -> u64 {
        let n = self.0;
        self.0 += 1;
        n
    }
}

#[derive(Debug, Clone)]
struct ToSend(Message, SocketAddr);

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
            let socket = UdpFramed::new(socket, CoapCodec);
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
            let msg = match msg {
                Msg::Valid(msg) => msg,
                Msg::Invalid(header, err) => {
                    warn!(
                        "Received CoAP message with invalid format: {}",
                        error::pprint_error(&err)
                    );
                    warn!("!UNIMPLEMENTED! Should send matching Reset message");
                    let rst = MessageBuilder::reset(header.message_id).build();
                    let snd = self
                        .tx
                        .clone()
                        .send(ToSend(rst, src))
                        .map(|_| ())
                        .map_err(|_| ());
                    tokio::spawn(snd);
                    continue;
                }
            };

            println!("{:?}", msg);
            println!("{}", msg);

            match msg.kind {
                MessageKind::Request(_) => {
                    let req = Request::from_message(src, msg).unwrap();
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
