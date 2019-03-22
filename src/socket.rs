use crate::codec::{CoapCodec, ParsedMsg};
use crate::error::{Error as CoapError, Result as CoapResult};
use crate::message::Message;
use futures::sink::SendAll;
use futures::stream::{iter_ok, IterOk};
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::vec::IntoIter as VecIntoIter;
use tokio::net::{UdpFramed, UdpSocket};
use tokio::prelude::stream::{SplitSink, SplitStream};
use tokio::prelude::*;

type RawUdp = UdpFramed<CoapCodec>;
type RawUdpIn = SplitStream<RawUdp>;
type RawUdpOut = SplitSink<RawUdp>;
type Outgoing = (Message, SocketAddr);
type SendingFut = SendAll<RawUdpOut, IterOk<VecIntoIter<Outgoing>, CoapError>>;

pub struct CoapSocket {
    outgoing: Vec<Outgoing>,
    sock_in: RawUdpIn,
    sock_out: Option<RawUdpOut>,
    sending: Option<SendingFut>,
}

impl CoapSocket {
    pub fn new<A: ToSocketAddrs>(addr: A) -> CoapResult<Self> {
        for addr in addr.to_socket_addrs()? {
            let socket = match UdpSocket::bind(&addr) {
                Ok(socket) => socket,
                Err(_) => continue,
            };
            let (sock_out, sock_in) = UdpFramed::new(socket, CoapCodec).split();
            return Ok(Self {
                outgoing: Vec::new(),
                sock_in,
                sock_out: Some(sock_out),
                sending: None,
            });
        }
        return Err(CoapError::addr_unavailable());
    }

    pub fn send(&mut self, msg: Message, dest: SocketAddr) {
        self.outgoing.push((msg, dest));
        task::current().notify();
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
                // println!("sending {:?}", out);
                let sock = self.sock_out.take().expect("unreachable");
                self.sending = Some(sock.send_all(iter_ok(out)));
            } else {
                return Ok(Async::NotReady);
            }
        }
    }

    pub fn poll_recv(&mut self) -> Poll<(ParsedMsg, SocketAddr), CoapError> {
        match self.sock_in.poll()? {
            Async::Ready(Some(inc)) => Ok(Async::Ready(inc)),
            Async::Ready(None) => panic!("what"),
            Async::NotReady => Ok(Async::NotReady),
        }
    }
}
