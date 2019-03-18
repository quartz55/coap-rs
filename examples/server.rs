#[allow(unused_imports)]
use coap::message::code::{ClientErrorCode, ServerErrorCode, SuccessCode};
use coap::message::{MessageBuilder, MessageType, Method, ResponseCode};
use coap::server::Server;
use futures::future;
use std::io;
use tokio::prelude::*;

fn main() {
    // let m1 = MessageBuilder::ping(0).build();
    // let m2 = MessageBuilder::reset(0).build();
    // let m3 = MessageBuilder::request()
    //     .confirmable()
    //     .message_id(12321)
    //     .method(Method::Post)
    //     .build();
    // let m4 = MessageBuilder::response()
    //     .acknowledgement()
    //     .message_id(m3.header.message_id)
    //     .response_code(SuccessCode::Created.into())
    //     .build();

    // println!("{}", m1);
    // println!("{}", m2);
    // println!("{}", m3);
    // println!("{}", m4);
    // return;

    env_logger::init();
    let addr = "0.0.0.0:5683";
    let server = Server::new(addr).unwrap();
    tokio::run(server.map_err(|e| println!("Server error = {:?}", e)));
}
