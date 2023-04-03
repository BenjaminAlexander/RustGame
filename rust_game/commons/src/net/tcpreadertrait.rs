use std::net::SocketAddr;
use serde::de::DeserializeOwned;
use serde::Serialize;
use rmp_serde::decode::Error as DecodeError;

pub trait TcpReaderTrait: Send + Sized {

}