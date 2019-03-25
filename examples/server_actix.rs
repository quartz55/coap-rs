#![allow(unused_imports)]
use actix::prelude::*;
use coap::message::code::{ClientErrorCode, ServerErrorCode, SuccessCode};
use coap::message::{MessageBuilder, MessageType, Method, ResponseCode};
use coap::server_actix::Server;
use futures::Future;

fn main() {
    env_logger::init();
    actix::System::run(|| {
        let _server = Server::start("0.0.0.0:5683").unwrap();
    });
}
