use std::io::Error;
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::net::tcpstreamtrait::TcpStreamTrait;

pub trait TcpListenerTrait: Send {
    type ReadType: Serialize + DeserializeOwned;
    type WriteType: Serialize + DeserializeOwned;
    type TcpStream: TcpStreamTrait<ReadType=Self::ReadType, WriteType=Self::WriteType>;

    fn accept(&self) -> Result<Self::TcpStream, Error>;
}