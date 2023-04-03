use std::net::SocketAddr;
use rmp_serde::decode::Error as DecodeError;
use serde::de::DeserializeOwned;
use serde::Serialize;
use commons::factory::FactoryTrait;
use commons::net::TcpReaderTrait;
use commons::threading::channel::Receiver;
use crate::singlethreaded::SingleThreadedFactory;

pub struct ChannelTcpReader {
    local_addr: SocketAddr,
    peer_addr: SocketAddr,
    receiver: Receiver<SingleThreadedFactory, Vec<u8>>
}

impl ChannelTcpReader {
    pub fn new(local_addr: SocketAddr, peer_addr: SocketAddr, receiver: Receiver<SingleThreadedFactory, Vec<u8>>) -> Self {
        return Self {
            local_addr,
            peer_addr,
            receiver
        }
    }
}

impl TcpReaderTrait for ChannelTcpReader {

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