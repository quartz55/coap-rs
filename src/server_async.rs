use crate::message::packet::Packet;
use crate::message::request::{CoAPRequest, Method};
use crate::message::response::CoAPResponse;
use futures::try_ready;
use log::{debug, error, info, warn};
use std::io;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use tokio::prelude::*;
use tokio::sync::mpsc;

use tokio::net::UdpSocket;

fn handler(request: CoAPRequest) -> impl Future<Item = Option<CoAPResponse>, Error = io::Error> {
    // match request.get_method() {
    //     Method::Get => println!("GET {}", request.get_path()),
    //     Method::Post => println!("POST {}", request.get_path()),
    //     _ => println!("ANY {}", request.get_path()),
    // }
    futures::future::ok(request.response)
}

struct Response(SocketAddr, Vec<u8>);

pub struct CoAPServer {
    socket: UdpSocket,
    buf: Vec<u8>,
    rx: mpsc::Receiver<Response>,
    tx: mpsc::Sender<Response>,
    to_send: Option<Response>,
}

impl CoAPServer {
    pub fn new<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
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
            });
        }
        return Err(io::ErrorKind::AddrNotAvailable.into());
    }
}

impl Future for CoAPServer {
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
            let packet = match Packet::from_bytes(&self.buf[..size]) {
                Ok(packet) => packet,
                Err(err) => {
                    warn!("Invalid CoAP packet received in socket: {:?}", err);
                    continue;
                }
            };
            let request = CoAPRequest::from_packet(packet, &addr);
            debug!("Received CoAP request from {}: {:?}", addr, request);
            let tx = self.tx.clone();
            let response_handler = handler(request)
                .map_err(|err| {
                    warn!("Request handler error: {:?}", err);
                    ()
                })
                .and_then(move |res| match res {
                    Some(res) => match res.message.to_bytes() {
                        Ok(bytes) => tx.send(Response(addr, bytes)).then(|res| match res {
                            Ok(_) => Ok(()),
                            Err(err) => panic!("{:?}", err),
                        }),
                        Err(err) => {
                            warn!("Invalid response {:?}", err);
                            panic!(":(");
                            // Ok(())
                        }
                    },
                    None => {
                        panic!(":(");
                        // Ok(())
                    }
                });
            tokio::spawn(response_handler);
        }
    }
}
