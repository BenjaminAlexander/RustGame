use std::io::Error;
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::ip::tcpstreamtrait::TcpStreamTrait;

pub trait TcpListenerTrait: Send {
    type TcpStream<ReadType: Serialize + DeserializeOwned + Send, WriteType: Serialize + DeserializeOwned + Send>: TcpStreamTrait<ReadType, WriteType>;

    fn accept<ReadType: Serialize + DeserializeOwned + Send, WriteType: Serialize + DeserializeOwned + Send>(&self) -> Result<Self::TcpStream<ReadType, WriteType>, Error>;
}