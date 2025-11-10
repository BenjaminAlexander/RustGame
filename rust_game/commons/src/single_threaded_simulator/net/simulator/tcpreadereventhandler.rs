use crate::net::TcpReadHandlerTrait;
use crate::threading::channel::ReceiveMetaData;
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

    fn read(mut self, buf: Vec<u8>) -> EventHandleResult<Self> {
        return match rmp_serde::from_read::<Cursor<Vec<u8>>, T::ReadType>(Cursor::new(buf)) {
            Ok(read) => match self.read_handler.on_read(read) {
                Continue(read_handler) => {
                    self.read_handler = read_handler;
                    EventHandleResult::TryForNextEvent(self)
                }
                Break(()) => EventHandleResult::StopThread,
            },
            Err(_) => {
                self.read_handler.on_read_error();
                EventHandleResult::StopThread
            }
        };
    }
}

impl<T: TcpReadHandlerTrait> EventHandlerTrait for TcpReaderEventHandler<T> {
    type Event = Vec<u8>;

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> EventHandleResult<Self> {
        match channel_event {
            ChannelEvent::ReceivedEvent(_, buf) => self.read(buf),
            ChannelEvent::Timeout => EventHandleResult::TryForNextEvent(self),
            ChannelEvent::ChannelEmpty => EventHandleResult::WaitForNextEvent(self),
            ChannelEvent::ChannelDisconnected => {
                self.read_handler.on_channel_disconnected();
                EventHandleResult::StopThread
            }
        }
    }

    fn on_stop(self, receive_meta_data: ReceiveMetaData) {
        self.read_handler.on_stop(receive_meta_data);
    }
}
