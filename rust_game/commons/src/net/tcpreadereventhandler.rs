use std::net::TcpListener;
use std::ops::ControlFlow;
use std::ops::ControlFlow::{Break, Continue};
use log::{error, warn};
use rmp_serde::decode::Error;
use crate::net::realtcpstream::RealTcpStream;
use crate::net::tcpconnectionhandlertrait::TcpConnectionHandlerTrait;
use crate::net::tcpreadhandlertrait::TcpReadHandlerTrait;
use crate::net::{TcpReaderTrait, TcpSenderTrait};
use crate::threading::channel::ReceiveMetaData;
use crate::threading::eventhandling::{ChannelEvent, ChannelEventResult, EventHandlerTrait};
use crate::threading::eventhandling::WaitOrTryForNextEvent::TryForNextEvent;

pub struct TcpReaderEventHandler<T: TcpReadHandlerTrait> {
    tcp_reader: RealTcpStream,
    tcp_read_handler: T
}

impl<T: TcpReadHandlerTrait> TcpReaderEventHandler<T> {

    pub fn new(tcp_reader: RealTcpStream, tcp_read_handler: T) -> Self {
        return Self {
            tcp_reader,
            tcp_read_handler
        };
    }

    fn read(mut self) -> ChannelEventResult<Self> {

        match self.tcp_reader.read::<T::ReadType>() {
            Ok(read_value) => {
                return match self.tcp_read_handler.on_read(read_value) {
                    Continue(()) => Continue(TryForNextEvent(self)),
                    Break(()) =>  Break(self.tcp_read_handler)
                };
            }
            Err(error) => {
                warn!("Error on TCP read: {:?}", error);
                return Break(self.tcp_read_handler);
            }
        }
    }
}

impl<T: TcpReadHandlerTrait> EventHandlerTrait for TcpReaderEventHandler<T> {
    type Event = ();
    type ThreadReturn = T;

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> ChannelEventResult<Self> {
        return match channel_event {
            ChannelEvent::ReceivedEvent(_, ()) => Continue(TryForNextEvent(self)),
            ChannelEvent::Timeout => Continue(TryForNextEvent(self)),
            ChannelEvent::ChannelEmpty => self.read(),
            ChannelEvent::ChannelDisconnected => Break(self.tcp_read_handler)
        };
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn {
        return self.tcp_read_handler;
    }
}