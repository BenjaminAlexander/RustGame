use crate::singlethreaded::SingleThreadedSender;
use commons::net::TcpWriterTrait;
use commons::threading::channel::SenderTrait;
use rmp_serde::encode::Error as EncodeError;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;

pub struct ChannelTcpWriter {
    peer_addr: SocketAddr,
    has_been_closed: bool,
    sender: SingleThreadedSender<Vec<u8>>,
}

impl ChannelTcpWriter {
    pub fn new(peer_addr: SocketAddr, sender: SingleThreadedSender<Vec<u8>>) -> Self {
        return Self {
            peer_addr,
            has_been_closed: false,
            sender,
        };
    }
}

impl TcpWriterTrait for ChannelTcpWriter {
    fn write<T: Serialize + DeserializeOwned>(&mut self, write: &T) -> Result<(), EncodeError> {
        let vec = rmp_serde::encode::to_vec(write)?;

        return match self.sender.send(vec) {
            Ok(()) => Ok(()),
            Err(_error) => {
                self.has_been_closed = true;
                Err(EncodeError::Syntax("Channel has been closed".to_string()))
            }
        };
    }

    fn flush(&mut self) -> Result<(), Error> {
        if self.has_been_closed {
            return Err(Error::from(ErrorKind::NotConnected));
        } else {
            return Ok(());
        }
    }

    fn get_peer_addr(&self) -> &SocketAddr {
        return &self.peer_addr;
    }
}
