use rmp_serde::encode::Error as EncodeError;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::Error;
use std::net::SocketAddr;

pub trait TcpWriterTrait: Send + Sized {
    fn write<T: Serialize + DeserializeOwned>(&mut self, write: &T) -> Result<(), EncodeError>;

    fn flush(&mut self) -> Result<(), Error>;

    fn get_peer_addr(&self) -> &SocketAddr;
}
