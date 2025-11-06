use super::constants::NET_POLLING_PERIOD;
use crate::net::tcp::RealTcpStream;
use crate::net::tcpconnectionhandlertrait::TcpConnectionHandlerTrait;
use crate::net::{TcpReceiver, TcpStream};
use crate::threading::channel::ReceiveMetaData;
use crate::threading::eventhandling::{
    ChannelEvent,
    EventHandleResult,
    EventHandlerTrait,
};
use log::error;
use std::io::{
    self,
    Error,
};
use std::net::{
    SocketAddr,
    TcpListener,
};
use std::ops::ControlFlow::{
    Break,
    Continue,
};

pub struct TcpListenerEventHandler<T: TcpConnectionHandlerTrait> {
    tcp_listener: TcpListener,
    tcp_connection_handler: T
}

impl<T: TcpConnectionHandlerTrait> TcpListenerEventHandler<T> {
    pub fn new(tcp_listener: TcpListener, tcp_connection_handler: T) -> io::Result<Self> {
        tcp_listener.set_nonblocking(true)?;

        return Ok(Self {
            tcp_listener,
            tcp_connection_handler
        });
    }

    fn accept(self) -> EventHandleResult<Self> {
        let accept_result = self.tcp_listener.accept();
        return self.handle_accept_result(accept_result);
    }

    fn handle_accept_result(
        self,
        accept_result: Result<(std::net::TcpStream, SocketAddr), Error>,
    ) -> EventHandleResult<Self> {
        match accept_result {
            Ok((net_tcp_stream, remote_peer_socket_addr)) => {
                let real_tcp_stream = RealTcpStream::new(net_tcp_stream, remote_peer_socket_addr);
                let tcp_stream_clone_result = real_tcp_stream.try_clone();
                return self
                    .handle_tcp_stream_clone_result(real_tcp_stream, tcp_stream_clone_result);
            }
            Err(ref error) if error.kind() == io::ErrorKind::WouldBlock => {
                return EventHandleResult::WaitForNextEventOrTimeout(self, NET_POLLING_PERIOD);
            }
            Err(error) => {
                error!("Error while trying to accept a TCP connection: {:?}", error);
                return EventHandleResult::TryForNextEvent(self);
            }
        }
    }

    fn handle_tcp_stream_clone_result(
        mut self,
        real_tcp_stream: RealTcpStream,
        clone_result: Result<RealTcpStream, Error>,
    ) -> EventHandleResult<Self> {
        match clone_result {
            Ok(real_tcp_stream_clone) => {
                match self
                    .tcp_connection_handler
                    .on_connection(TcpStream::new(real_tcp_stream), TcpReceiver::new(real_tcp_stream_clone))
                {
                    Continue(()) => {
                        return EventHandleResult::TryForNextEvent(self);
                    }
                    Break(()) => {
                        return EventHandleResult::StopThread(self.tcp_connection_handler);
                    }
                }
            }
            Err(_) => {
                error!(
                    "Failed to clone RealTcpStream for : {:?}",
                    real_tcp_stream.get_peer_addr()
                );
                return EventHandleResult::TryForNextEvent(self);
            }
        }
    }
}

impl<T: TcpConnectionHandlerTrait> EventHandlerTrait for TcpListenerEventHandler<T> {
    type Event = ();
    type ThreadReturn = T;

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> EventHandleResult<Self> {
        return match channel_event {
            ChannelEvent::ReceivedEvent(_, ()) => EventHandleResult::TryForNextEvent(self),
            ChannelEvent::Timeout => self.accept(),
            ChannelEvent::ChannelEmpty => self.accept(),
            ChannelEvent::ChannelDisconnected => {
                EventHandleResult::StopThread(self.tcp_connection_handler)
            }
        };
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn {
        return self.tcp_connection_handler;
    }
}

#[cfg(test)]
mod tests {
    use std::net::TcpStream;

    use io::ErrorKind;
    use log::LevelFilter;

    use super::*;
    use crate::{
        logging::LoggingConfigBuilder,
        net::{
            TcpConnectionHandler,
            LOCAL_EPHEMERAL_SOCKET_ADDR_V4,
        },
    };

    #[test]
    fn test_handle_tcp_stream_clone_result() {
        LoggingConfigBuilder::new()
            .add_console_appender()
            .init(LevelFilter::Info);

        let tcp_connection_handler = TcpConnectionHandler::new();

        let tcp_listener = TcpListener::bind(LOCAL_EPHEMERAL_SOCKET_ADDR_V4).unwrap();

        let listener_addr = tcp_listener.local_addr().unwrap();

        let tcp_stream = TcpStream::connect(listener_addr).unwrap();

        let real_tcp_stream = RealTcpStream::new(tcp_stream, listener_addr);

        let event_handler =
            TcpListenerEventHandler::new(tcp_listener, tcp_connection_handler).unwrap();

        let event_handler_result = event_handler.handle_tcp_stream_clone_result(
            real_tcp_stream,
            Result::Err(Error::from(ErrorKind::NotConnected)),
        );

        assert!(matches!(
            event_handler_result,
            EventHandleResult::TryForNextEvent(_)
        ));
    }

    #[test]
    fn test_handle_accept_result() {
        LoggingConfigBuilder::new()
            .add_console_appender()
            .init(LevelFilter::Info);

        let tcp_connection_handler = TcpConnectionHandler::new();

        let tcp_listener = TcpListener::bind(LOCAL_EPHEMERAL_SOCKET_ADDR_V4).unwrap();

        let event_handler =
            TcpListenerEventHandler::new(tcp_listener, tcp_connection_handler).unwrap();

        let event_handler_result =
            event_handler.handle_accept_result(Result::Err(Error::from(ErrorKind::NotConnected)));

        assert!(matches!(
            event_handler_result,
            EventHandleResult::TryForNextEvent(_)
        ));
    }
}
