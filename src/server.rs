use crate::error::{self, Error as CoapError, ErrorKind, MessageError, Result as CoapResult};
use crate::message::{Message, MessageBuilder, MessageKind};
use crate::request::Request;
use crate::response::{Carry, Response};
use arrayvec::ArrayVec;
use futures::future::FutureResult;
use futures::future::{self, Either};
use futures::try_ready;
use log::{debug, error, info, warn};
use std::io;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use tokio::prelude::*;
use tokio::sync::mpsc;

use tokio::net::UdpSocket;

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

pub fn default_handler(request: &Request) -> impl Future<Item = Carry, Error = ()> {
    future::ok(Carry::Piggyback(Response::from_request(request)))
}

pub trait Handler {
    fn handle(&mut self, request: &Request) -> Box<dyn Future<Item = Carry, Error = ()> + Send>;
}

impl<B, F> Handler for F
where
    B: IntoFuture<Item = Carry, Error = ()>,
    B::Future: Send + 'static,
    F: Fn(&Request) -> B,
{
    fn handle(&mut self, request: &Request) -> Box<dyn Future<Item = Carry, Error = ()> + Send> {
        return Box::new(self(request).into_future());
    }
}

pub struct Server<H> {
    socket: UdpSocket,
    buf: Vec<u8>,
    rx: mpsc::Receiver<Carry>,
    tx: mpsc::Sender<Carry>,
    to_send: Option<Carry>,
    handler: H,
    mid: MidGen,
}

impl<H> Server<H> {
    pub fn with_handler<A: ToSocketAddrs>(addr: A, handler: H) -> io::Result<Self> {
        for addr in addr.to_socket_addrs()? {
            let socket = match UdpSocket::bind(&addr) {
                Ok(socket) => socket,
                Err(_) => continue,
            };
            let (tx, rx) = mpsc::channel(1024);
            return Ok(Self {
                socket,
                buf: vec![0u8; 1024],
                rx,
                tx,
                to_send: None,
                handler,
                mid: MidGen::new(),
            });
        }
        return Err(io::ErrorKind::AddrNotAvailable.into());
    }
}

impl<H> Future for Server<H>
// where
//     H: Handler,
{
    type Item = ();
    type Error = CoapError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            // Check for requests
            let (size, addr) = try_ready!(self
                .socket
                .poll_recv_from(&mut self.buf)
                .map_err(|e| ErrorKind::ServerIo(e)));

            debug!("Got {} bytes from {:?}", size, addr);
            let message = match Message::from_bytes(&self.buf[..size]) {
                Ok(msg) => msg,
                Err(MessageError::PacketTooSmall(_)) => {
                    warn!("Received non CoAP datagram");
                    continue;
                }
                Err(MessageError::MessageFormat(err)) => {
                    warn!(
                        "Received CoAP message with invalid format: {}",
                        error::pprint_error(&err)
                    );
                    warn!("!UNIMPLEMENTED! Should send matching Reset message");
                    continue;
                }
            };
            println!("{:?}", message);
            println!("{}", message);

            match message.kind {
                MessageKind::Request(_) => {
                    let req = Request::from_message(addr, message).unwrap();
                }
                MessageKind::Empty => {}
                MessageKind::Response(_) => {
                    warn!("what?");
                }
                MessageKind::Reserved(_) => {
                    warn!("Ignoring message using reserved codes");
                }
            }

            // let tx = self.tx.clone();
            // tokio::spawn(
            //     tx.send(Carry::Piggyback(Response::from_request(&req)))
            //         .map(|_| ())
            //         .map_err(|_| ()),
            // );
        }
    }
}
