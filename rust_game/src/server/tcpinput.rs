use std::net::TcpStream;

use log::{error, info, trace};
use rmp_serde::decode::Error;

use crate::interface::{Input, InputEvent};
use crate::messaging::{ToServerMessageTCP, InputMessage};
use crate::threading::{ChannelThread, Consumer, ConsumerList, Receiver, Sender};
use crate::gametime::{TimeReceived, TimeValue};
use std::io;
use crate::threading::sender::SendError;
use serde::export::PhantomData;

pub struct TcpInput {
    tcp_stream: TcpStream
}

impl TcpInput {

    pub fn new(tcp_stream: &TcpStream) -> io::Result<Self> {
        Ok(Self {tcp_stream: tcp_stream.try_clone()?})
    }
}

impl ChannelThread<()> for TcpInput {

    fn run(mut self, receiver: Receiver<Self>) {
        info!("Starting");

        let receiver = receiver;

        loop {
            let result: Result<ToServerMessageTCP, Error> = rmp_serde::from_read(&self.tcp_stream);

            //TODO: check player ID on message

            match result {
                Ok(message) => {
                    let time_received = TimeValue::now();

                    receiver.try_iter(&mut self);

                    match message {

                    }
                }
                Err(error) => {
                    error!("Ending due to: {:?}", error);
                    return;
                }
            }

            receiver.try_iter(&mut self);
        }
    }
}