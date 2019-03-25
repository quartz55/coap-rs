use crate::codec::{CoapCodec, ParsedMsg};
use crate::error::{self, Error as CoapError};
use crate::message::{Message as CoapMessage, MessageBuilder, MessageKind};
use crate::request::Request as CoapRequest;
use actix::prelude::*;
use futures::Future;
use std::collections::HashSet;
use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use tokio::net::{UdpFramed, UdpSocket};
use tokio::prelude::stream::{SplitSink, SplitStream};
use tokio::prelude::*;

#[derive(Message, Clone)]
struct Incoming(pub ParsedMsg, pub SocketAddr);

#[derive(Message)]
struct Send(pub CoapMessage, pub SocketAddr);

struct UdpManager {
    out: Option<SplitSink<UdpFramed<CoapCodec>>>,
    server: Addr<Server>,
}

impl StreamHandler<(ParsedMsg, SocketAddr), CoapError> for UdpManager {
    fn handle(&mut self, (msg, src): (ParsedMsg, SocketAddr), _ctx: &mut Context<Self>) {
        self.server.do_send(Incoming(msg, src));
    }
}

impl Handler<Send> for UdpManager {
    type Result = ();
    fn handle(&mut self, Send(msg, src): Send, ctx: &mut Context<Self>) {
        ctx.wait(
            self.out
                .take()
                .unwrap()
                .send((msg, src))
                .into_actor(self)
                .map(|out, act, _ctx| {
                    act.out = Some(out);
                })
                .map_err(|_, _, ctx| ctx.stop()),
        );
    }
}

impl Actor for UdpManager {
    type Context = Context<Self>;
}

pub struct Server {
    addr: SocketAddr,
    manager: Addr<UdpManager>,
}

impl Server {
    pub fn start<A: ToSocketAddrs>(addr: A) -> Result<Addr<Self>, CoapError> {
        let socket = get_socket(addr)?;
        let addr = socket.local_addr().unwrap();
        let (out, inc) = UdpFramed::new(socket, CoapCodec).split();
        Ok(Self::create(move |server| {
            let server = server.address();
            let manager = UdpManager::create(|udp| {
                udp.add_stream(inc);
                UdpManager {
                    out: Some(out),
                    server,
                }
            });
            Self { addr, manager }
        }))
    }

    fn send(&self, msg: CoapMessage, dest: SocketAddr) {
        self.manager.do_send(Send(msg, dest))
    }
}

impl Actor for Server {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("CoAP server listening at {}", self.addr);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        println!("Server stopped");
    }
}

impl Handler<Incoming> for Server {
    type Result = ();
    fn handle(&mut self, Incoming(pmsg, src): Incoming, ctx: &mut Context<Self>) {
        let msg = match pmsg {
            ParsedMsg::Valid(msg) => msg,
            ParsedMsg::Reject(header, err) => {
                warn!(
                    "Rejecting CoAP message with invalid format: {}",
                    error::pprint_error(&err)
                );
                let rst = MessageBuilder::reset(header.message_id()).build();
                self.send(rst, src);
                return;
            }
            ParsedMsg::Invalid(err) => {
                warn!(
                    "Silently ignoring invalid CoAP message: {}",
                    error::pprint_error(&err)
                );
                return;
            }
        };

        debug!("Incoming message\n{0:?}\n{0}", msg);

        if msg.is_reserved() {
            warn!("Silently ignoring message using reserved code:\n{}", msg);
            return;
        }

        match msg.kind() {
            MessageKind::Request(_) => {
                let req = CoapRequest::from_message(src, msg).unwrap();
            }
            _ => {}
        }
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
