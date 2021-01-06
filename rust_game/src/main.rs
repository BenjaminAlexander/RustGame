use std::{thread, time};
use log::info;
use crate::messaging::*;
use crate::simplegame::Vector2;
use crate::threading::ChannelThread;
use crate::gametime::TimeDuration;

mod simplegame;
mod messaging;
mod server;
mod logging;
mod threading;
mod interface;
mod gametime;
mod util;
mod client;

pub fn insert(v: &mut Vec<i32>, val: i32) {

    match v.binary_search_by(|elem| { val.cmp(elem) }) {
        Ok(pos) => v[pos] = val,
        Err(pos) => v.insert(pos, val)
    }
}

pub fn main() {

    logging::init_logging();

    info!("Hello, world!");

    let mut v = Vec::<i32>::new();

    insert(&mut v, 6);
    insert(&mut v, 4);
    insert(&mut v, 5);

    info!("test {:?}", v);
    info!("test {:?}", v.pop().unwrap());
    info!("test {:?}", v);

    let input_message:InputMessage<Vector2> = InputMessage::new(0, 0, Vector2::new(1.0, 12.0));
    let _my_message:ToServerMessage<Vector2> = ToServerMessage::Input(input_message);

    let server_core  = server::Core::<Vector2, Vector2>::new(3456, TimeDuration::from_millis(50));
    let (server_core_sender, server_core_builder) = server_core.build();

    server_core_sender.start_listener();
    server_core_builder.name("ServerCore").start().unwrap();

    let client_core = client::Core::<Vector2, Vector2>::new(
        "127.0.0.1",
        3456,
        TimeDuration::from_millis(50),
        50);

    let (client_core_sender, client_core_builder) = client_core.build();

    client_core_sender.connect();
    client_core_builder.name("ClientCore").start().unwrap();

    let millis = time::Duration::from_millis(1000);
    thread::sleep(millis);

    server_core_sender.start_game();

    // let mut stream = TcpStream::connect("127.0.0.1:3456").unwrap();
    // rmp_serde::encode::write(&mut stream, &my_message).unwrap();
    // stream.flush().unwrap();

    let ten_millis = time::Duration::from_millis(10000);
    thread::sleep(ten_millis);
}