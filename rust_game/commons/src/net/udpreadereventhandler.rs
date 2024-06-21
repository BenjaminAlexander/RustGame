use crate::net::udpreadhandlertrait::UdpReadHandlerTrait;
use crate::net::RealUdpSocket;
use crate::threading::channel::ReceiveMetaData;
use crate::threading::eventhandling::{ChannelEvent, EventHandleResult, EventHandlerTrait};
use log::warn;
use std::ops::ControlFlow::{Break, Continue};

pub const MAX_UDP_DATAGRAM_SIZE: usize = 1500;

pub struct UdpReaderEventHandler<T: UdpReadHandlerTrait> {
    udp_socket: RealUdpSocket,
    udp_read_handler: T,
}

impl<T: UdpReadHandlerTrait> UdpReaderEventHandler<T> {
    pub fn new(udp_socket: RealUdpSocket, udp_read_handler: T) -> Self {
        return Self {
            udp_socket,
            udp_read_handler,
        };
    }

    fn read(mut self) -> EventHandleResult<Self> {
        let mut buf = [0; MAX_UDP_DATAGRAM_SIZE];

        match self.udp_socket.recv_from(&mut buf) {
            Ok((len, peer_addr)) => {
                return match self.udp_read_handler.on_read(peer_addr, &buf[..len]) {
                    Continue(()) => EventHandleResult::TryForNextEvent(self),
                    Break(()) => EventHandleResult::StopThread(self.udp_read_handler),
                };
            }
            Err(error) => {
                warn!("Error on UDP read: {:?}", error);
                return EventHandleResult::TryForNextEvent(self);
            }
        }
    }
}

impl<T: UdpReadHandlerTrait> EventHandlerTrait for UdpReaderEventHandler<T> {
    type Event = ();
    type ThreadReturn = T;

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> EventHandleResult<Self> {
        return match channel_event {
            ChannelEvent::ReceivedEvent(_, ()) => EventHandleResult::TryForNextEvent(self),
            ChannelEvent::Timeout => EventHandleResult::TryForNextEvent(self),
            ChannelEvent::ChannelEmpty => self.read(),
            ChannelEvent::ChannelDisconnected => {
                EventHandleResult::StopThread(self.udp_read_handler)
            }
        };
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn {
        return self.udp_read_handler;
    }
}
