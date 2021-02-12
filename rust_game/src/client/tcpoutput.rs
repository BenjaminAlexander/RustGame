use log::{trace, info};
use std::net::TcpStream;
use crate::threading::{ChannelDrivenThread, Consumer, Sender, ChannelThread, Receiver};
use crate::interface::{Input, InputEvent};
use crate::messaging::{InputMessage, ToServerMessageTCP};
use std::io;
use std::marker::PhantomData;
use std::io::Write;

//TODO: Send response to time messages to calculate ping

pub struct TcpOutput {
    tcp_stream: TcpStream
}

impl TcpOutput {

    pub fn new(tcp_stream: &TcpStream) -> io::Result<Self> {
        return Ok(Self{
            tcp_stream: tcp_stream.try_clone()?
        });
    }
}

impl ChannelThread<()> for TcpOutput {

    fn run(mut self, receiver: Receiver<Self>) -> () {
        loop {
            trace!("Waiting.");
            match receiver.recv(&mut self) {
                Err(_error) => {
                    info!("Channel closed.");
                    return ();
                }
                _ => {}
            }

            let mut send_another_message = true;
            while send_another_message {
                receiver.try_iter(&mut self);

                send_another_message = false;
            }
        }
    }
}