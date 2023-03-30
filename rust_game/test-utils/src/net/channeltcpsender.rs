use std::any::Any;
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use rmp_serde::encode::Error as EncodeError;
use rmp_serde::encode::Error::InvalidValueWrite;
use serde::de::DeserializeOwned;
use serde::Serialize;
use commons::factory::FactoryTrait;
use commons::net::TcpSenderTrait;
use commons::threading::channel::{SendError, SenderTrait};

pub struct ChannelTcpStream<Factory: FactoryTrait> {
    peer_addr: SocketAddr,
    has_been_closed: bool,
    sender: Factory::Sender<Box<Vec<u8>>>
}

impl<Factory: FactoryTrait> TcpSenderTrait for ChannelTcpStream<Factory> {
    fn write<T: Serialize + DeserializeOwned>(&mut self, write: &T) -> Result<(), EncodeError> {

        let vec = rmp_serde::encode::to_vec(write)?;

        return match self.sender.send(Box::new(vec)) {
            Ok(()) => Ok(()),
            Err(e) => {
                self.has_been_closed = true;
                Err(EncodeError::Syntax("Channel has been closed".to_string()))
            }
        };
    }

    fn flush(&mut self) -> Result<(), Error> {
        if self.has_been_closed {
            return Err( Error::from(ErrorKind::NotConnected));
        } else {
            return Ok(());
        }
    }

    fn get_peer_addr(&self) -> &SocketAddr {
        return &self.peer_addr;
    }
}