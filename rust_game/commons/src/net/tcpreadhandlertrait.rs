use std::ops::ControlFlow;
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::net::{TcpReaderTrait, TcpSenderTrait};

pub trait TcpReadHandlerTrait: Send + 'static {
    type ReadType: Serialize + DeserializeOwned;

    fn on_read(&mut self, read: Self::ReadType) -> ControlFlow<()>;
}