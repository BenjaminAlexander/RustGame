use std::io::Cursor;
use std::ops::ControlFlow::{Break, Continue};
use commons::net::TcpReadHandlerTrait;
use commons::threading::channel::ReceiveMetaData;
use commons::threading::eventhandling::{ChannelEvent, ChannelEventResult, EventHandlerTrait};
use commons::threading::eventhandling::WaitOrTryForNextEvent::{TryForNextEvent, WaitForNextEvent};

pub struct TcpReaderEventHandler<T: TcpReadHandlerTrait> {
    read_handler: T
}

impl<T: TcpReadHandlerTrait> TcpReaderEventHandler<T> {

    pub fn new(read_handler: T) -> Self {
        return Self {
            read_handler
        };
    }

    fn read(mut self, buf: Vec<u8>) -> ChannelEventResult<Self> {
        return match rmp_serde::from_read::<Cursor<Vec<u8>>, T::ReadType>(Cursor::new(buf)) {
            Ok(read) => match self.read_handler.on_read(read) {
                    Continue(()) => Continue(TryForNextEvent(self)),
                    Break(()) => Break(self.read_handler)
                }
            Err(_) => Break(self.read_handler)
        };
    }
}

impl<T: TcpReadHandlerTrait> EventHandlerTrait for TcpReaderEventHandler<T> {
    type Event = Vec<u8>;
    type ThreadReturn = T;

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> ChannelEventResult<Self> {
        match channel_event {
            ChannelEvent::ReceivedEvent(_, buf) => self.read(buf),
            ChannelEvent::Timeout => Continue(TryForNextEvent(self)),
            ChannelEvent::ChannelEmpty => Continue(WaitForNextEvent(self)),
            ChannelEvent::ChannelDisconnected => Break(self.read_handler)
        }
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn {
        return self.read_handler;
    }
}