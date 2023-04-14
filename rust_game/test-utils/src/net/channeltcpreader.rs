use std::net::SocketAddr;
use rmp_serde::decode::Error as DecodeError;
use serde::de::DeserializeOwned;
use serde::Serialize;
use commons::threading::channel::{Receiver, TryRecvError};
use crate::singlethreaded::{SingleThreadedFactory, SingleThreadedSender};

pub struct ChannelTcpReader {
    sender: SingleThreadedSender<Vec<u8>>,
    receiver: Receiver<SingleThreadedFactory, Vec<u8>>
}

impl ChannelTcpReader {
    pub fn new(sender: SingleThreadedSender<Vec<u8>>, receiver: Receiver<SingleThreadedFactory, Vec<u8>>) -> Self {
        return Self {
            sender,
            receiver
        }
    }

    pub fn take(self) -> (SingleThreadedSender<Vec<u8>>, Receiver<SingleThreadedFactory, Vec<u8>>) {
        return (self.sender, self.receiver);
    }

    pub fn try_read(&mut self) -> Result<Vec<u8>, TryRecvError> {
        match self.receiver.try_recv() {
            Ok(buf) =>  Ok(buf),
            Err(error) =>  Err(error)
        }
    }
}