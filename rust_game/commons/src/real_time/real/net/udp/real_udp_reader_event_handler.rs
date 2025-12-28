use {
    crate::real_time::{
        EventHandleResult,
        HandleEvent,
        ReceiveMetaData,
        event_or_stop_thread::EventOrStopThread,
        net::{
            MAX_UDP_DATAGRAM_SIZE,
            udp::HandleUdpRead,
        },
        real::{
            self,
            RealReceiver,
            net::udp::RealUdpSocket,
        },
    },
    log::warn,
    std::{
        io::{
            Error,
            ErrorKind,
        },
        ops::ControlFlow::{
            Break,
            Continue,
        },
    },
};

pub struct RealUdpReaderEventHandler<T: HandleUdpRead> {
    udp_socket: RealUdpSocket,
    udp_read_handler: T,
}

impl<T: HandleUdpRead> RealUdpReaderEventHandler<T> {
    pub fn spawn_udp_reader(
        thread_name: String,
        receiver: RealReceiver<EventOrStopThread<()>>,
        udp_socket: RealUdpSocket,
        udp_read_handler: T,
        join_call_back: impl FnOnce(()) + Send + 'static,
    ) -> Result<(), Error> {
        let event_handler = Self {
            udp_socket,
            udp_read_handler,
        };

        return real::spawn_event_handler(thread_name, receiver, event_handler, join_call_back);
    }

    fn read(&mut self) -> EventHandleResult {
        let mut buf = [0; MAX_UDP_DATAGRAM_SIZE];
        let result = self.udp_socket.recv_from(&mut buf);
        return self.handle_read_result(result, &buf);
    }

    fn handle_read_result(
        &mut self,
        result: Result<(usize, std::net::SocketAddr), std::io::Error>,
        buf: &[u8],
    ) -> EventHandleResult {
        match result {
            Ok((len, peer_addr)) => {
                return match self.udp_read_handler.on_read(peer_addr, &buf[..len]) {
                    Continue(()) => EventHandleResult::TryForNextEvent,
                    Break(()) => EventHandleResult::StopThread,
                };
            }
            Err(error)
                if error.kind() == ErrorKind::TimedOut || error.kind() == ErrorKind::WouldBlock =>
            {
                return EventHandleResult::TryForNextEvent;
            }
            Err(error) => {
                warn!("Error on UDP read: {:?}", error);
                return EventHandleResult::TryForNextEvent;
            }
        }
    }
}

impl<T: HandleUdpRead> HandleEvent for RealUdpReaderEventHandler<T> {
    type Event = ();
    type ThreadReturn = ();

    fn on_event(&mut self, _: ReceiveMetaData, _: Self::Event) -> EventHandleResult {
        return EventHandleResult::TryForNextEvent;
    }

    fn on_timeout(&mut self) -> EventHandleResult {
        return EventHandleResult::TryForNextEvent;
    }

    fn on_channel_empty(&mut self) -> EventHandleResult {
        return self.read();
    }

    fn on_stop_self(self) -> Self::ThreadReturn {
        return ();
    }
}

#[cfg(test)]
mod tests {

    use {
        super::*,
        crate::{
            logging::LoggingConfigBuilder,
            real_time::net::LOCAL_EPHEMERAL_SOCKET_ADDR_V4,
        },
        log::LevelFilter,
        std::{
            io::Error,
            net::SocketAddr,
            ops::ControlFlow,
        },
    };

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

        let mut read_handler = RealUdpReaderEventHandler {
            udp_socket,
            udp_read_handler,
        };

        let buf = [0];

        assert_eq!(
            ControlFlow::Continue(()),
            udp_read_handler(SocketAddr::from(LOCAL_EPHEMERAL_SOCKET_ADDR_V4), &buf)
        );

        let result = Result::Err(Error::from(ErrorKind::NotConnected));

        let handle_result_result = read_handler.handle_read_result(result, &buf);

        assert!(matches!(
            handle_result_result,
            EventHandleResult::TryForNextEvent
        ));
    }
}
