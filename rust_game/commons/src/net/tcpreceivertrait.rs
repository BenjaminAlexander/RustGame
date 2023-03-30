use std::net::SocketAddr;
use serde::de::DeserializeOwned;
use serde::Serialize;
use rmp_serde::decode::Error as DecodeError;

pub trait TcpReceiverTrait: Send + Sized {

    fn read<T: Serialize + DeserializeOwned>(&self) -> Result<T, DecodeError>;

    fn get_peer_addr(&self) -> &SocketAddr;
}