use std::io::Error;
use std::net::SocketAddr;
use serde::de::DeserializeOwned;
use serde::Serialize;
use rmp_serde::decode::Error as DecodeError;
use rmp_serde::encode::Error as EncodeError;

pub trait TcpStreamTrait: Send + Sized {
    type ReadType: Serialize + DeserializeOwned;
    type WriteType: Serialize + DeserializeOwned;

    fn read(&self) -> Result<Self::ReadType, DecodeError>;

    fn write(&mut self, write: &Self::WriteType) -> Result<(), EncodeError>;

    fn flush(&mut self) -> Result<(), Error>;

    fn get_peer_addr(&self) -> &SocketAddr;

    fn try_clone(&self) -> Result<Self, Error>;
}