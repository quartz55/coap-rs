use coap::{server_async::CoAPServer, CoAPRequest, CoAPResponse, IsMessage, Method};
use std::io;
use tokio::prelude::*;

fn main() {
    env_logger::init();
    let addr = "0.0.0.0:5683";

    let mut server = CoAPServer::new(addr).unwrap();
    tokio::run(server.map_err(|e| println!("Server error = {:?}", e)));
}
