mod simplegame;
mod messaging;
mod server;
mod runnablethread;
mod logging;
mod threading;

use crate::simplegame::Vector2;
use crate::messaging::*;
use crate::server::*;
use rmp_serde::*;
use std::net::{TcpStream, SocketAddr};
use log::{SetLoggerError, LevelFilter};
use crate::runnablethread::{RunnableThread, MessageHandler};
use std::{thread, time};
use log::{info, warn, error};
use std::io::{BufWriter, Write, LineWriter};
use crate::threading::{Thread, ChannelThread};

struct SimpleHandler;

impl MessageHandler<fn()> for SimpleHandler{
    fn handle(&self, message: fn()) {
        message();
    }
}

pub fn main() {

    logging::initLogging();

    info!("Hello, world!");

    let inputMessage:InputMessage<Vector2> = InputMessage::new(0, 0, Vector2::new(1.0, 12.0));
    let myMessage:ToServerMessage<Vector2> = ToServerMessage::Input(inputMessage);


    let (coreSender, coreThreadBuilder) =
        Core::<Vector2, Vector2>::new().build();

    let listenerBuilder = TcpListenerThread::new(3456, coreSender).build();

    coreThreadBuilder.name("ServerCore".to_string()).start().unwrap();
    listenerBuilder.name("TcpListener".to_string()).start().unwrap();

    let millis = time::Duration::from_millis(500);
    thread::sleep(millis);

    let mut stream = TcpStream::connect("127.0.0.1:3456").unwrap();
    rmp_serde::encode::write(&mut stream, &myMessage).unwrap();
    stream.flush();

    let ten_millis = time::Duration::from_millis(5000);
    thread::sleep(ten_millis);
}