#[allow(unused_imports)]
use coap::message::code::{ClientErrorCode, ServerErrorCode, SuccessCode};
use coap::message::{MessageBuilder, MessageType, Method, ResponseCode};
use coap::server::{default_handler, Server};
use futures::future;
use std::io;
use tokio::prelude::*;

// fn handler(request: CoAPRequest) -> impl Future<Item = Option<CoAPResponse>, Error = ()> {
// match request.get_method() {
//     Method::Get => println!("GET {}", request.get_path()),
//     Method::Post => println!("POST {}", request.get_path()),
//     _ => println!("ANY {}", request.get_path()),
// }
// let now = std::time::Instant::now();
// tokio::timer::Delay::new(now + std::time::Duration::from_millis(1000))
//     .and_then(move |_| {
//         let elapsed = now.elapsed();
//         let ms = elapsed.as_millis();
//         let mut res = request.response.take().unwrap();
//         let orig_pl = String::from_utf8(res.message.payload.clone()).unwrap();
//         res.message
//             .set_payload(format!("{} ({} ms)", orig_pl, ms).into_bytes());
//         future::ok(Some(res))
//     })
//     .map_err(|e| eprintln!("{:?}", e))
//     future::ok(request.response)
// }

// struct Handler {
//     ctx: u32,
// }

// impl coap::server_async::Handler for Handler {
//     fn handle(
//         &mut self,
//         mut request: CoAPRequest,
//     ) -> Box<dyn Future<Item = Option<CoAPResponse>, Error = ()> + Send> {
//         let now = std::time::Instant::now();
//         let mut res = request.response.take().unwrap();
//         res.message
//             .set_payload(format!("[{:?}] ctx={}", now, self.ctx).into_bytes());
//         self.ctx += 1;
//         Box::new(future::ok(Some(res)))
//     }
// }

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
    let server = Server::with_handler(addr, default_handler).unwrap();
    tokio::run(server.map_err(|e| println!("Server error = {:?}", e)));
}
