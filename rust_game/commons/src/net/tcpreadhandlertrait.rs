use serde::de::DeserializeOwned;
use serde::Serialize;
use std::ops::ControlFlow;

use crate::threading::channel::ReceiveMetaData;

pub trait TcpReadHandlerTrait: Send + Sized + 'static {
    type ReadType: Serialize + DeserializeOwned;

    fn on_read(self, read: Self::ReadType) -> ControlFlow<(), Self>;

    //TODO: this needs some documentation
    fn on_channel_disconnected(self) {
        //no-op default implementation
    }

    //TODO: this needs some documentation
    fn on_read_error(self) {
        //no-op default implementation
    }

    //TODO: this needs some documentation
    fn on_stop(self, _receive_meta_data: ReceiveMetaData) {
        //no-op default implementation
    }
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

    fn on_read(mut self, read: Self::ReadType) -> ControlFlow<(), Self> {
        match (self.on_read)(read) {
            ControlFlow::Continue(()) => ControlFlow::Continue(self),
            ControlFlow::Break(()) => ControlFlow::Break(()),
        }
    }
}
