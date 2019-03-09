use crate::message::message::Message;
use crate::message::packet::Packet;
use crate::message::request::{CoAPRequest, Method};
use crate::message::response::CoAPResponse;
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

fn default_handler(request: CoAPRequest) -> impl Future<Item = Option<CoAPResponse>, Error = ()> {
    future::ok(request.response)
}

pub trait Handler {
    fn handle(
        &mut self,
        request: CoAPRequest,
    ) -> Box<dyn Future<Item = Option<CoAPResponse>, Error = ()> + Send>;
}

impl<B, F> Handler for F
where
    B: IntoFuture<Item = Option<CoAPResponse>, Error = ()>,
    B::Future: Send + 'static,
    F: Fn(CoAPRequest) -> B,
{
    fn handle(
        &mut self,
        request: CoAPRequest,
    ) -> Box<dyn Future<Item = Option<CoAPResponse>, Error = ()> + Send> {
        return Box::new(self(request).into_future());
    }
}

struct Response(SocketAddr, Vec<u8>);

pub struct CoAPServer<H> {
    socket: UdpSocket,
    buf: Vec<u8>,
    rx: mpsc::Receiver<Response>,
    tx: mpsc::Sender<Response>,
    to_send: Option<Response>,
    handler: H,
}

impl<H> CoAPServer<H> {
    pub fn with_handler<A: ToSocketAddrs>(addr: A, handler: H) -> io::Result<Self> {
        for addr in addr.to_socket_addrs()? {
            let socket = match UdpSocket::bind(&addr) {
                Ok(socket) => socket,
                Err(_) => continue,
            };
            let (tx, rx) = mpsc::channel(1024);
            return Ok(Self {
                socket,
                buf: vec![0; 1024],
                rx,
                tx,
                to_send: None,
                handler,
            });
        }
        return Err(io::ErrorKind::AddrNotAvailable.into());
    }
}

impl<H> Future for CoAPServer<H>
where
    H: Handler,
{
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            // Handle response to send
            if let Some(Response(ref addr, ref bytes)) = self.to_send {
                let amt = try_ready!(self.socket.poll_send_to(bytes, addr));
                debug!("Sent {} bytes of response to {:?}", amt, addr);
                self.to_send = None;
            };

            match self.rx.poll() {
                Ok(Async::NotReady) => {}
                Ok(Async::Ready(Some(res))) => {
                    self.to_send = Some(res);
                    continue;
                }
                Ok(Async::Ready(None)) | Err(_) => return Err(io::ErrorKind::BrokenPipe.into()),
            };

            // Check for requests
            let (size, addr) = try_ready!(self.socket.poll_recv_from(&mut self.buf));
            debug!("Got {} bytes from {:?}", size, addr);
            let message = match Message::from_bytes(&self.buf[..size]) {
                Ok(msg) => msg,
                Err(err) => {
                    warn!("Invalid CoAP packet received in socket: {:?}", err);
                    continue;
                }
            };
            println!("{:?}", message);
            match message.payload {
                Some(pl) => println!("{}", String::from_utf8(pl).unwrap()),
                None => {}
            };
            // let packet = match Packet::from_bytes(&self.buf[..size]) {
            //     Ok(packet) => packet,
            //     Err(err) => {
            //         warn!("Invalid CoAP packet received in socket: {:?}", err);
            //         continue;
            //     }
            // };
            // let request = CoAPRequest::from_packet(packet, &addr);
            // debug!("Received CoAP request from {}: {:?}", addr, request);
            // let tx = self.tx.clone();
            // let response_handler = self
            //     .handler
            //     .handle(request)
            //     .map_err(|err| {
            //         warn!("Request handler error: {:?}", err);
            //         ()
            //     })
            //     .and_then(move |res| match res {
            //         Some(res) => match res.message.to_bytes() {
            //             Ok(bytes) => Either::A(
            //                 tx.send(Response(addr, bytes))
            //                     .map(|_| ())
            //                     .map_err(|_| panic!("boom")),
            //             ),
            //             Err(err) => {
            //                 warn!("Invalid response {:?}", err);
            //                 Either::B(future::ok(()))
            //             }
            //         },
            //         None => {
            //             debug!("No response");
            //             Either::B(future::ok(()))
            //         }
            //     });
            // tokio::spawn(response_handler);
        }
    }
}
