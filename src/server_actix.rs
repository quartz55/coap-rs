use crate::codec::{CoapCodec, ParsedMsg};
use crate::error::Error as CoapError;
use actix::prelude::*;
use futures::Future;
use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use tokio::net::{UdpFramed, UdpSocket};
use tokio::prelude::stream::{SplitSink, SplitStream};
use tokio::prelude::*;

#[derive(Message)]
struct Incoming(pub ParsedMsg, pub SocketAddr);

struct UdpManager;
impl Handler<Incoming> for UdpManager {
    type Result = ();

    fn handle(&mut self, Incoming(msg, src): Incoming, ctx: &mut Context<Self>) {}
}
impl Actor for UdpManager {
    type Context = Context<Self>;
}

pub struct Server {
    manager: Addr<UdpManager>,
}

impl Server {
    pub fn new<A: ToSocketAddrs>(addr: A) -> Result<Self, CoapError> {
        let socket = get_socket(addr)?;
        let (out, inc) = UdpFramed::new(socket, CoapCodec).split();
        let manager = UdpManager::create(|ctx| {
            ctx.add_message_stream(inc.map_err(|_| ()).map(|(m, s)| Incoming(m, s)));
            UdpManager
        });
        Ok(Self { manager })
    }
}

impl Actor for Server {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Server started");
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        println!("Server stopped");
    }
}

fn get_socket<A: ToSocketAddrs>(addr: A) -> Result<UdpSocket, CoapError> {
    for addr in addr.to_socket_addrs()? {
        match UdpSocket::bind(&addr) {
            Ok(socket) => return Ok(socket),
            Err(_) => continue,
        };
    }
    return Err(CoapError::addr_unavailable());
}
