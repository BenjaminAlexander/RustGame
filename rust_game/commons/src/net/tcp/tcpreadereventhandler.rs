use crate::net::tcp::resetablereader::{
    DeserializeResult,
    ResetableReader,
};
use crate::net::tcpreadhandlertrait::TcpReadHandlerTrait;
use crate::threading::channel::ReceiveMetaData;
use crate::threading::eventhandling::{
    ChannelEvent,
    EventHandleResult,
    EventHandlerTrait,
};
use std::ops::ControlFlow;

use crate::net::tcp::RealTcpStream;

pub struct TcpReaderEventHandler<T: TcpReadHandlerTrait> {
    tcp_resetable_reader: ResetableReader<std::net::TcpStream>,
    tcp_read_handler: T,
}

impl<T: TcpReadHandlerTrait> TcpReaderEventHandler<T> {
    pub fn new(tcp_reader: RealTcpStream, tcp_read_handler: T) -> Self {
        return Self {
            tcp_resetable_reader: ResetableReader::new(tcp_reader.take_std_net_tcp_reader()),
            tcp_read_handler,
        };
    }

    fn read(&mut self) -> EventHandleResult<Self> {
        match self.tcp_resetable_reader.deserialize::<T::ReadType>() {
            DeserializeResult::Ok(read_value) => {
                return match self.tcp_read_handler.on_read(read_value) {
                    ControlFlow::Continue(()) => EventHandleResult::TryForNextEvent,
                    ControlFlow::Break(()) => EventHandleResult::StopThread(()),
                };
            }
            DeserializeResult::TimedOut => EventHandleResult::TryForNextEvent,
            DeserializeResult::Err => EventHandleResult::StopThread(()),
        }
    }
}

impl<T: TcpReadHandlerTrait> EventHandlerTrait for TcpReaderEventHandler<T> {
    type Event = ();
    type ThreadReturn = ();

    fn on_channel_event(
        &mut self,
        channel_event: ChannelEvent<Self::Event>,
    ) -> EventHandleResult<Self> {
        return match channel_event {
            ChannelEvent::ChannelEmpty => self.read(),
            ChannelEvent::ChannelDisconnected => EventHandleResult::StopThread(()),
            _ => EventHandleResult::TryForNextEvent,
        };
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn {
        return ();
    }
}
