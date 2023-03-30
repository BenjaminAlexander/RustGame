use std::net::SocketAddr;
use rmp_serde::decode::Error as DecodeError;
use serde::de::DeserializeOwned;
use serde::Serialize;
use commons::factory::FactoryTrait;
use commons::net::TcpReceiverTrait;
use commons::threading::channel::Receiver;

pub struct ChannelTcpReceiver/*<Factory: FactoryTrait>*/ {
    //peer_addr: SocketAddr,
    //receiver: Receiver<Box<Vec<u8>>>
}
/*
impl<Factory: FactoryTrait> TcpReceiverTrait for ChannelTcpReceiver<Factory> {

    fn read<T: Serialize + DeserializeOwned>(&self) -> Result<T, DecodeError> {
        todo!()
    }

    fn get_peer_addr(&self) -> &SocketAddr {
        todo!()
    }
}
 */