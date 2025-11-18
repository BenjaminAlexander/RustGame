use crate::real_time::event_or_stop_thread::EventOrStopThread;
use crate::real_time::net::tcp::{
    TcpConnectionHandlerTrait,
    TcpReader,
    TcpStream,
};
use crate::real_time::net::NET_POLLING_PERIOD;
use crate::real_time::real::net::tcp::RealTcpStream;
use crate::real_time::real::RealReceiver;
use crate::real_time::{
    real,
    EventHandleResult,
    HandleEvent,
    ReceiveMetaData,
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
    tcp_connection_handler: T,
}

impl<T: TcpConnectionHandlerTrait> TcpListenerEventHandler<T> {
    pub fn spawn_tcp_listener(
        thread_name: String,
        real_receiver: RealReceiver<EventOrStopThread<()>>,
        socket_addr: SocketAddr,
        mut tcp_connection_handler: T,
        join_call_back: impl FnOnce(()) + Send + 'static,
    ) -> std::io::Result<()> {
        let tcp_listener = TcpListener::bind(socket_addr)?;
        tcp_listener.set_nonblocking(true)?;

        tcp_connection_handler.on_bind(tcp_listener.local_addr()?);

        let event_handler = Self {
            tcp_listener,
            tcp_connection_handler,
        };

        return real::spawn_event_handler(
            thread_name,
            real_receiver,
            event_handler,
            join_call_back,
        );
    }

    fn accept(&mut self) -> EventHandleResult {
        let accept_result = self.tcp_listener.accept();
        return self.handle_accept_result(accept_result);
    }

    fn handle_accept_result(
        &mut self,
        accept_result: Result<(std::net::TcpStream, SocketAddr), Error>,
    ) -> EventHandleResult {
        match accept_result {
            Ok((net_tcp_stream, remote_peer_socket_addr)) => {
                let real_tcp_stream = RealTcpStream::new(net_tcp_stream, remote_peer_socket_addr);
                let tcp_stream_clone_result = real_tcp_stream.try_clone();
                return self
                    .handle_tcp_stream_clone_result(real_tcp_stream, tcp_stream_clone_result);
            }
            Err(ref error) if error.kind() == io::ErrorKind::WouldBlock => {
                return EventHandleResult::WaitForNextEventOrTimeout(NET_POLLING_PERIOD);
            }
            Err(error) => {
                error!("Error while trying to accept a TCP connection: {:?}", error);
                return EventHandleResult::TryForNextEvent;
            }
        }
    }

    fn handle_tcp_stream_clone_result(
        &mut self,
        real_tcp_stream: RealTcpStream,
        clone_result: Result<RealTcpStream, Error>,
    ) -> EventHandleResult {
        match clone_result {
            Ok(real_tcp_stream_clone) => {
                match self.tcp_connection_handler.on_connection(
                    TcpStream::new(real_tcp_stream),
                    TcpReader::new(real_tcp_stream_clone),
                ) {
                    Continue(()) => {
                        return EventHandleResult::TryForNextEvent;
                    }
                    Break(()) => {
                        return EventHandleResult::StopThread;
                    }
                }
            }
            Err(_) => {
                error!(
                    "Failed to clone RealTcpStream for : {:?}",
                    real_tcp_stream.get_peer_addr()
                );
                return EventHandleResult::TryForNextEvent;
            }
        }
    }
}

impl<T: TcpConnectionHandlerTrait> HandleEvent for TcpListenerEventHandler<T> {
    type Event = ();
    type ThreadReturn = ();

    fn on_event(&mut self, _: ReceiveMetaData, _: Self::Event) -> EventHandleResult {
        return EventHandleResult::TryForNextEvent;
    }

    fn on_timeout(&mut self) -> EventHandleResult {
        return self.accept();
    }

    fn on_channel_empty(&mut self) -> EventHandleResult {
        return self.accept();
    }

    fn on_stop_self(self) -> Self::ThreadReturn {
        return ();
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
        real_time::net::{
            tcp::TcpConnectionHandler,
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

        let mut event_handler = TcpListenerEventHandler {
            tcp_listener,
            tcp_connection_handler,
        };

        let event_handler_result = event_handler.handle_tcp_stream_clone_result(
            real_tcp_stream,
            Result::Err(Error::from(ErrorKind::NotConnected)),
        );

        assert!(matches!(
            event_handler_result,
            EventHandleResult::TryForNextEvent
        ));
    }

    #[test]
    fn test_handle_accept_result() {
        LoggingConfigBuilder::new()
            .add_console_appender()
            .init(LevelFilter::Info);

        let tcp_connection_handler = TcpConnectionHandler::new();

        let tcp_listener = TcpListener::bind(LOCAL_EPHEMERAL_SOCKET_ADDR_V4).unwrap();

        let mut event_handler = TcpListenerEventHandler {
            tcp_listener,
            tcp_connection_handler,
        };

        let event_handler_result =
            event_handler.handle_accept_result(Result::Err(Error::from(ErrorKind::NotConnected)));

        assert!(matches!(
            event_handler_result,
            EventHandleResult::TryForNextEvent
        ));
    }
}
