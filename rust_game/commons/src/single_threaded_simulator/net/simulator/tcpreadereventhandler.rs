use crate::net::TcpReadHandlerTrait;
use crate::real_time::ReceiveMetaData;
use crate::threading::eventhandling::{
    ChannelEvent,
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

    fn on_channel_event(
        &mut self,
        channel_event: ChannelEvent<Self::Event>,
    ) -> EventHandleResult<Self> {
        match channel_event {
            ChannelEvent::ReceivedEvent(_, buf) => self.read(buf),
            ChannelEvent::Timeout => EventHandleResult::TryForNextEvent,
            ChannelEvent::ChannelEmpty => EventHandleResult::WaitForNextEvent,
            ChannelEvent::ChannelDisconnected => EventHandleResult::StopThread(()),
        }
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn {
        return ();
    }
}
