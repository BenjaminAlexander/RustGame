use crate::net::realtcpstream::RealTcpStream;
use crate::net::tcpreadhandlertrait::TcpReadHandlerTrait;
use crate::threading::channel::ReceiveMetaData;
use crate::threading::eventhandling::{
    ChannelEvent,
    EventHandleResult,
    EventHandlerTrait,
};
use std::ops::ControlFlow::{
    self, Break, Continue
};

use super::realtcpstream::TcpDeserializer;

pub struct TcpReaderEventHandler<T: TcpReadHandlerTrait> {
    tcp_deserializer: TcpDeserializer,
    tcp_read_handler: T,
}

impl<T: TcpReadHandlerTrait> TcpReaderEventHandler<T> {
    pub fn new(tcp_reader: RealTcpStream, tcp_read_handler: T) -> Self {
        return Self {
            tcp_deserializer: tcp_reader.to_deserializer(),
            tcp_read_handler,
        };
    }

    fn read(mut self) -> EventHandleResult<Self> {
        match self.tcp_deserializer.read::<T::ReadType>() {
            Ok(read_value) => {
                return match self.tcp_read_handler.on_read(read_value) {
                    Continue(()) => EventHandleResult::TryForNextEvent(self),
                    Break(()) => EventHandleResult::StopThread(self.tcp_read_handler),
                };
            }
            Err(ControlFlow::Continue(())) => EventHandleResult::TryForNextEvent(self),
            Err(ControlFlow::Break(())) =>  EventHandleResult::StopThread(self.tcp_read_handler)
        }
    }
}

impl<T: TcpReadHandlerTrait> EventHandlerTrait for TcpReaderEventHandler<T> {
    type Event = ();
    type ThreadReturn = T;

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> EventHandleResult<Self> {
        return match channel_event {
            ChannelEvent::ReceivedEvent(_, ()) => EventHandleResult::TryForNextEvent(self),
            ChannelEvent::Timeout => EventHandleResult::TryForNextEvent(self),
            ChannelEvent::ChannelEmpty => self.read(),
            ChannelEvent::ChannelDisconnected => {
                EventHandleResult::StopThread(self.tcp_read_handler)
            }
        };
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn {
        return self.tcp_read_handler;
    }
}
