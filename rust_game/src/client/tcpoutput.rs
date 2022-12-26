use log::info;
use std::net::TcpStream;
use crate::threading::{ChannelThread, Receiver, ThreadAction};
use std::io;

//TODO: Send response to time messages to calculate ping
//Should this e a channel driven thread?
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

impl ChannelThread<(), ThreadAction> for TcpOutput {

    fn run(mut self, receiver: Receiver<Self, ThreadAction>) -> () {
        loop {
            match receiver.recv(&mut self) {
                Ok(ThreadAction::Continue) => {}
                Ok(ThreadAction::Stop) => {
                    info!("Thread commanded to stop.");
                    return;
                }
                Err(error) => {
                    info!("Thread stopped due to disconnect: {:?}", error);
                    return;
                }
            }
        }
    }
}