use serde::de::DeserializeOwned;
use serde::Serialize;
use std::ops::ControlFlow;

pub trait TcpReadHandlerTrait: Send + 'static {
    type ReadType: Serialize + DeserializeOwned;

    fn on_read(&mut self, read: Self::ReadType) -> ControlFlow<()>;
}

pub struct TcpReadHandler<T: Serialize + DeserializeOwned + 'static> {
    on_read: Box<dyn FnMut(T) -> ControlFlow<()> + Send + 'static>,
}

impl<T: Serialize + DeserializeOwned + 'static> TcpReadHandler<T> {
    pub fn new(on_read: impl FnMut(T) -> ControlFlow<()> + Send + 'static) -> Self {
        return TcpReadHandler {
            on_read: Box::new(on_read),
        };
    }
}

impl<T: Serialize + DeserializeOwned + 'static> TcpReadHandlerTrait for TcpReadHandler<T> {
    type ReadType = T;

    fn on_read(&mut self, read: Self::ReadType) -> ControlFlow<()> {
        return (self.on_read)(read);
    }
}
