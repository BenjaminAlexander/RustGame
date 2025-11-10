use crate::net::udpreadhandlertrait::UdpReadHandlerTrait;
use crate::net::RealUdpSocket;
use crate::threading::channel::ReceiveMetaData;
use crate::threading::eventhandling::{
    ChannelEvent,
    EventHandleResult,
    EventHandlerTrait,
};
use log::warn;
use std::io::ErrorKind;
use std::ops::ControlFlow::{
    Break,
    Continue,
};

use super::MAX_UDP_DATAGRAM_SIZE;

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
        let result = self.udp_socket.recv_from(&mut buf);
        return self.handle_read_result(result, &buf);
    }

    fn handle_read_result(
        mut self,
        result: Result<(usize, std::net::SocketAddr), std::io::Error>,
        buf: &[u8],
    ) -> EventHandleResult<Self> {
        match result {
            Ok((len, peer_addr)) => {
                return match self.udp_read_handler.on_read(peer_addr, &buf[..len]) {
                    Continue(udp_read_handler) => {
                        self.udp_read_handler = udp_read_handler;
                        EventHandleResult::TryForNextEvent(self)
                    }
                    Break(()) => EventHandleResult::StopThread,
                };
            }
            Err(error)
                if error.kind() == ErrorKind::TimedOut || error.kind() == ErrorKind::WouldBlock =>
            {
                return EventHandleResult::TryForNextEvent(self);
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

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> EventHandleResult<Self> {
        return match channel_event {
            ChannelEvent::ChannelEmpty => self.read(),
            ChannelEvent::ChannelDisconnected => {
                self.udp_read_handler.on_channel_disconnected();
                EventHandleResult::StopThread
            }
            _ => EventHandleResult::TryForNextEvent(self),
        };
    }

    fn on_stop(self, receive_meta_data: ReceiveMetaData) {
        self.udp_read_handler.on_stop(receive_meta_data);
    }
}

#[cfg(test)]
mod tests {

    use std::{
        io::Error,
        net::SocketAddr,
        ops::ControlFlow,
    };

    use log::LevelFilter;

    use crate::{
        logging::LoggingConfigBuilder,
        net::LOCAL_EPHEMERAL_SOCKET_ADDR_V4,
    };

    use super::*;

    #[test]
    fn test_handle_read_result() {
        LoggingConfigBuilder::new()
            .add_console_appender()
            .init(LevelFilter::Info);

        let udp_socket =
            RealUdpSocket::bind(SocketAddr::from(LOCAL_EPHEMERAL_SOCKET_ADDR_V4)).unwrap();

        let udp_read_handler = move |_, _: &[u8]| {
            return ControlFlow::Continue(());
        };

        let read_handler = UdpReaderEventHandler::new(udp_socket, udp_read_handler);

        let buf = [0];

        assert_eq!(
            ControlFlow::Continue(()),
            udp_read_handler(SocketAddr::from(LOCAL_EPHEMERAL_SOCKET_ADDR_V4), &buf)
        );

        let result = Result::Err(Error::from(ErrorKind::NotConnected));

        let handle_result_result = read_handler.handle_read_result(result, &buf);

        assert!(matches!(
            handle_result_result,
            EventHandleResult::TryForNextEvent(_)
        ));
    }
}
