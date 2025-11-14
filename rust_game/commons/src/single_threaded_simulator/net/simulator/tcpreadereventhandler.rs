use crate::net::TcpReadHandlerTrait;
use crate::real_time::ReceiveMetaData;
use crate::threading::eventhandling::{
    EventHandleResult,
    EventHandlerTrait,
};
use std::io::Cursor;
use std::ops::ControlFlow::{
    Break,
    Continue,
};

pub struct TcpReaderEventHandler<T: TcpReadHandlerTrait> {
    read_handler: T,
}

impl<T: TcpReadHandlerTrait> TcpReaderEventHandler<T> {
    pub fn new(read_handler: T) -> Self {
        return Self { read_handler };
    }

    fn read(&mut self, buf: Vec<u8>) -> EventHandleResult<Self> {
        return match rmp_serde::from_read::<Cursor<Vec<u8>>, T::ReadType>(Cursor::new(buf)) {
            Ok(read) => match self.read_handler.on_read(read) {
                Continue(()) => EventHandleResult::TryForNextEvent,
                Break(()) => EventHandleResult::StopThread(()),
            },
            Err(_) => EventHandleResult::StopThread(()),
        };
    }
}

impl<T: TcpReadHandlerTrait> EventHandlerTrait for TcpReaderEventHandler<T> {
    type Event = Vec<u8>;
    type ThreadReturn = ();

    fn on_stop(self, _: ReceiveMetaData) -> Self::ThreadReturn {
        return ();
    }
    
    fn on_event(&mut self, _: ReceiveMetaData, buf: Self::Event) -> EventHandleResult<Self> {
        self.read(buf)
    }
    
    //TODO: could this be the default wait?
    fn on_timeout(&mut self) -> EventHandleResult<Self> {
        EventHandleResult::TryForNextEvent
    }

    fn on_channel_disconnect(&mut self) -> EventHandleResult<Self> {
        EventHandleResult::StopThread(())
    }
}
