use std::io::Error;
use std::net::SocketAddr;
use serde::de::DeserializeOwned;
use serde::Serialize;
use rmp_serde::decode::Error as DecodeError;
use rmp_serde::encode::Error as EncodeError;

pub trait TcpStreamTrait: Send + Sized {

    fn read<T: Serialize + DeserializeOwned>(&self) -> Result<T, DecodeError>;

    fn write<T: Serialize + DeserializeOwned>(&mut self, write: &T) -> Result<(), EncodeError>;

    fn flush(&mut self) -> Result<(), Error>;

    fn get_peer_addr(&self) -> &SocketAddr;

    fn try_clone(&self) -> Result<Self, Error>;
}