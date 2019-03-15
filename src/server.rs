use crate::error::{self, Error as CoapError, ErrorKind, MessageError, Result as CoapResult};
use crate::message::Message;
use crate::message::MessageBuilder;
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
    buf: ArrayVec<[u8; 1024]>,
    rx: mpsc::Receiver<Carry>,
    tx: mpsc::Sender<Carry>,
    to_send: Option<Carry>,
    handler: H,
    mid: u64,
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
                buf: ArrayVec::new(),
                rx,
                tx,
                to_send: None,
                handler,
                mid: 0,
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
            if let Some(ref carry) = self.to_send {
                match carry {
                    Carry::Piggyback(res) => {
                        let addr = res.dest();
                        let message = res.serialize();
                        debug!("Trying to send message to {:?}\n{}", addr, message);
                        let out = message.as_bytes().unwrap();
                        let amt = try_ready!(self
                            .socket
                            .poll_send_to(&out, addr)
                            .map_err(|e| ErrorKind::ServerIo(e)));
                        debug!("Sent {} bytes of response to {:?}", amt, addr);
                        self.to_send = None;
                    }
                }
            };

            match self.rx.poll() {
                Ok(Async::NotReady) => {}
                Ok(Async::Ready(Some(res))) => {
                    self.to_send = Some(res);
                    continue;
                }
                Ok(Async::Ready(None)) | Err(_) => {
                    return Err(ErrorKind::ServerIo(io::ErrorKind::BrokenPipe.into()))?;
                }
            };

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
                Err(err @ MessageError::MessageFormat(_)) => {
                    warn!(
                        "Received CoAP message with invalid format: {}",
                        error::pprint_error(&err)
                    );
                    warn!("Should send matching Reset message");
                    continue;
                }
            };

            println!("{:?}", message);
            println!("{}", message);
            let req = Request::from_message(addr, message).unwrap();
            println!("{:?}", req);
            let ack = MessageBuilder::empty()
                .acknowledgement()
                .message_id(req.message_id())
                .build();
            try_ready!(self
                .socket
                .poll_send_to(&ack.as_bytes()?, &addr)
                .map_err(ErrorKind::ServerIo));
            let res = MessageBuilder::response()
                .message_id(req.message_id())
                .token(req.token().clone())
                .build();
            let tx = self.tx.clone();
            tokio::spawn(
                tx.send(Carry::Piggyback(Response::from_request(&req)))
                    .map(|_| ())
                    .map_err(|_| ()),
            );
        }
    }
}
