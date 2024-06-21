use serde::de::DeserializeOwned;
use serde::Serialize;
use std::ops::ControlFlow;

pub trait TcpReadHandlerTrait: Send + 'static {
    type ReadType: Serialize + DeserializeOwned;

    fn on_read(&mut self, read: Self::ReadType) -> ControlFlow<()>;
}
