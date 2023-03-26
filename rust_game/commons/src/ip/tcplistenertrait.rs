use std::io::Error;
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::ip::tcpstreamtrait::TcpStreamTrait;

pub trait TcpListenerTrait {
    type TcpStream<T: Serialize + DeserializeOwned>: TcpStreamTrait<T=T>;

    fn accept<T: Serialize + DeserializeOwned>(&self) -> Result<Self::TcpStream<T>, Error>;
}