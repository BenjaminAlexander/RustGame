use std::{thread, time};
use std::io::Write;
use std::net::TcpStream;

use log::info;

use crate::messaging::*;
use crate::server::*;
use crate::simplegame::Vector2;
use crate::threading::{ChannelThread, Thread};
use crate::gametime::GameTimer;

mod simplegame;
mod messaging;
mod server;
mod logging;
mod threading;
mod interface;
mod gametime;

pub fn main() {

    logging::init_logging();

    info!("Hello, world!");

    let (sender, builder) = GameTimer::new(chrono::Duration::milliseconds(50)).build();
    builder.name("Timer".to_string()).start().unwrap();
    sender.start();

    let input_message:InputMessage<Vector2> = InputMessage::new(0, 0, Vector2::new(1.0, 12.0));
    let my_message:ToServerMessage<Vector2> = ToServerMessage::Input(input_message);


    let (core_sender, core_thread_builder) =
        Core::<Vector2, Vector2>::new().build();

    let listener_builder = TcpListenerThread::new(3456, core_sender).build();

    core_thread_builder.name("ServerCore".to_string()).start().unwrap();
    listener_builder.name("TcpListener".to_string()).start().unwrap();

    let millis = time::Duration::from_millis(500);
    thread::sleep(millis);

    let mut stream = TcpStream::connect("127.0.0.1:3456").unwrap();
    rmp_serde::encode::write(&mut stream, &my_message).unwrap();
    stream.flush().unwrap();

    let ten_millis = time::Duration::from_millis(5000);
    thread::sleep(ten_millis);
}