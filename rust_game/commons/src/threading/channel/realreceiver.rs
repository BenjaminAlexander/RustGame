use crate::net::{
    RealTcpStream,
    RealUdpSocket,
    TcpConnectionHandlerTrait,
    TcpListenerEventHandler,
    TcpReadHandlerTrait,
    TcpReaderEventHandler,
    UdpReadHandlerTrait,
    UdpReaderEventHandler,
};
use crate::threading::channel::{
    ReceiveMetaData,
    ReceiverTrait,
    SendMetaData,
};
use crate::threading::eventhandling::{
    EventHandlerThread,
    EventHandlerTrait,
    EventOrStopThread,
};
use crate::time::{
    TimeDuration,
    TimeSource,
};
use std::io::Error;
use std::net::{
    SocketAddr,
    TcpListener,
};
use std::sync::mpsc::{
    self,
    TryRecvError,
};

pub type RecvError = mpsc::RecvError;

pub type RecvTimeoutError = mpsc::RecvTimeoutError;

pub struct RealReceiver<T: Send> {
    time_source: TimeSource,
    receiver: mpsc::Receiver<(SendMetaData, T)>, //duration_in_queue_logger: RollingStatsLogger<TimeDuration>
}

impl<T: Send> ReceiverTrait<T> for RealReceiver<T> {
    fn try_recv_meta_data(&mut self) -> Result<(ReceiveMetaData, T), TryRecvError> {
        let (send_meta_data, value) = self.receiver.try_recv()?;
        return Ok((self.make_receive_meta_data(send_meta_data), value));
    }
}

impl<T: Send> RealReceiver<T> {
    pub fn new(time_source: TimeSource, receiver: mpsc::Receiver<(SendMetaData, T)>) -> Self {
        return Self {
            time_source,
            receiver, //duration_in_queue_logger: RollingStatsLogger::new(100, 3.5, TimeDuration::from_seconds(30.0))
        };
    }

    pub fn recv_meta_data(&mut self) -> Result<(ReceiveMetaData, T), RecvError> {
        let (send_meta_data, value) = self.receiver.recv()?;
        return Ok((self.make_receive_meta_data(send_meta_data), value));
    }

    pub fn recv(&mut self) -> Result<T, RecvError> {
        let (_, value) = self.recv_meta_data()?;
        return Ok(value);
    }

    pub fn recv_timeout_meta_data(
        &mut self,
        duration: TimeDuration,
    ) -> Result<(ReceiveMetaData, T), RecvTimeoutError> {
        if let Some(std_duration) = duration.to_duration() {
            let (send_meta_data, value) = self.receiver.recv_timeout(std_duration)?;
            return Ok((self.make_receive_meta_data(send_meta_data), value));
        } else {
            return Err(RecvTimeoutError::Timeout);
        }
    }

    pub fn recv_timeout(&mut self, duration: TimeDuration) -> Result<T, RecvTimeoutError> {
        let (_, value) = self.recv_timeout_meta_data(duration)?;
        return Ok(value);
    }

    fn make_receive_meta_data(&mut self, send_meta_data: SendMetaData) -> ReceiveMetaData {
        let receive_meta_data = ReceiveMetaData::new(&self.time_source, send_meta_data);
        //self.duration_in_queue_logger.add_value(receive_meta_data.get_duration_in_queue());
        return receive_meta_data;
    }
}

impl<T: Send> RealReceiver<EventOrStopThread<T>> {
    //TODO: remove
    pub fn spawn_event_handler<U: EventHandlerTrait<Event = T>>(
        self,
        thread_name: String,
        event_handler: U,
        join_call_back: impl FnOnce(U::ThreadReturn) + Send + 'static,
    ) -> std::io::Result<()> {
        return EventHandlerThread::spawn_thread(thread_name, self, event_handler, join_call_back);
    }
}

impl RealReceiver<EventOrStopThread<()>> {
    //TODO: can these span methods be on a trait and called with dynamic dispatch?

    pub fn spawn_tcp_listener<T: TcpConnectionHandlerTrait>(
        self,
        thread_name: String,
        socket_addr: SocketAddr,
        mut tcp_connection_handler: T,
        join_call_back: impl FnOnce(T) + Send + 'static,
    ) -> std::io::Result<()> {
        let tcp_listener = TcpListener::bind(socket_addr)?;

        tcp_connection_handler.on_bind(tcp_listener.local_addr()?);

        let event_handler = TcpListenerEventHandler::new(tcp_listener, tcp_connection_handler)?;

        return self.spawn_event_handler(thread_name, event_handler, join_call_back);
    }

    pub fn spawn_real_tcp_reader<T: TcpReadHandlerTrait>(
        self,
        thread_name: String,
        real_tcp_stream: RealTcpStream,
        tcp_read_handler: T,
        join_call_back: impl FnOnce(T) + Send + 'static,
    ) -> Result<(), Error> {
        let event_handler = TcpReaderEventHandler::new(real_tcp_stream, tcp_read_handler);
        return self.spawn_event_handler(thread_name, event_handler, join_call_back);
    }

    pub fn spawn_real_udp_reader<T: UdpReadHandlerTrait>(
        self,
        thread_name: String,
        udp_socket: RealUdpSocket,
        udp_read_handler: T,
        join_call_back: impl FnOnce(T) + Send + 'static,
    ) -> Result<(), Error> {
        let event_handler = UdpReaderEventHandler::new(udp_socket, udp_read_handler);
        return self.spawn_event_handler(thread_name, event_handler, join_call_back);
    }
}
