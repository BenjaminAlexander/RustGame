use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use rmp_serde::encode::Error as EncodeError;
use serde::de::DeserializeOwned;
use serde::Serialize;
use commons::factory::FactoryTrait;
use commons::net::TcpSenderTrait;
use commons::threading::channel::SenderTrait;

pub struct ChannelTcpSender<Factory: FactoryTrait> {
    peer_addr: SocketAddr,
    has_been_closed: bool,
    sender: Factory::Sender<Vec<u8>>
}

impl<Factory: FactoryTrait> TcpSenderTrait for ChannelTcpSender<Factory> {
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
            return Err( Error::from(ErrorKind::NotConnected));
        } else {
            return Ok(());
        }
    }

    fn get_peer_addr(&self) -> &SocketAddr {
        return &self.peer_addr;
    }
}