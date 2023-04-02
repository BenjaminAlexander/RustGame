use std::net::SocketAddr;
use rmp_serde::decode::Error as DecodeError;
use serde::de::DeserializeOwned;
use serde::Serialize;
use commons::factory::FactoryTrait;
use commons::net::TcpReaderTrait;
use commons::threading::channel::Receiver;

pub struct ChannelTcpReader<Factory: FactoryTrait> {
    peer_addr: SocketAddr,
    receiver: Receiver<Factory, Vec<u8>>
}

impl<Factory: FactoryTrait> ChannelTcpReader<Factory> {
    pub fn new(peer_addr: SocketAddr, receiver: Receiver<Factory, Vec<u8>>) -> Self {
        return Self {
            peer_addr,
            receiver
        }
    }
}

impl<Factory: FactoryTrait> TcpReaderTrait for ChannelTcpReader<Factory> {

    fn read<T: Serialize + DeserializeOwned>(&mut self) -> Result<T, DecodeError> {
        match self.receiver.recv() {
            Ok(vec) =>  rmp_serde::from_slice(&vec[..]),
            Err(_) =>  Err(DecodeError::Syntax("Channel has been closed".to_string()))
        }
    }

    fn get_peer_addr(&self) -> &SocketAddr {
        return &self.peer_addr;
    }
}