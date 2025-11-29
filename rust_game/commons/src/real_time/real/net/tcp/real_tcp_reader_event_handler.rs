use crate::real_time::{
    event_or_stop_thread::EventOrStopThread,
    net::tcp::HandleTcpRead,
    real::{
        self,
        net::tcp::{
            resetable_reader::{
                DeserializeResult,
                ResetableReader,
            },
            RealTcpStream,
        },
        RealReceiver,
    },
    EventHandleResult,
    HandleEvent,
    ReceiveMetaData,
};
use std::{
    io::Error,
    ops::ControlFlow,
};

pub struct RealTcpReaderEventHandler<T: HandleTcpRead> {
    tcp_resetable_reader: ResetableReader<std::net::TcpStream>,
    tcp_read_handler: T,
}

impl<T: HandleTcpRead> RealTcpReaderEventHandler<T> {
    pub fn spawn_tcp_reader(
        thread_name: String,
        receiver: RealReceiver<EventOrStopThread<()>>,
        real_tcp_stream: RealTcpStream,
        tcp_read_handler: T,
        join_call_back: impl FnOnce(()) + Send + 'static,
    ) -> Result<(), Error> {
        let event_handler = Self {
            tcp_resetable_reader: ResetableReader::new(real_tcp_stream.take_std_net_tcp_reader()),
            tcp_read_handler,
        };

        return real::spawn_event_handler(thread_name, receiver, event_handler, join_call_back);
    }

    fn read(&mut self) -> EventHandleResult {
        match self.tcp_resetable_reader.deserialize::<T::ReadType>() {
            DeserializeResult::Ok(read_value) => {
                return match self.tcp_read_handler.on_read(read_value) {
                    ControlFlow::Continue(()) => EventHandleResult::TryForNextEvent,
                    ControlFlow::Break(()) => EventHandleResult::StopThread,
                };
            }
            DeserializeResult::TimedOut => EventHandleResult::TryForNextEvent,
            DeserializeResult::Err => EventHandleResult::StopThread,
        }
    }
}

impl<T: HandleTcpRead> HandleEvent for RealTcpReaderEventHandler<T> {
    type Event = ();
    type ThreadReturn = ();

    fn on_event(&mut self, _: ReceiveMetaData, _: Self::Event) -> EventHandleResult {
        return EventHandleResult::TryForNextEvent;
    }

    fn on_channel_empty(&mut self) -> EventHandleResult {
        return self.read();
    }

    fn on_timeout(&mut self) -> EventHandleResult {
        return EventHandleResult::TryForNextEvent;
    }

    fn on_stop_self(self) -> Self::ThreadReturn {
        return ();
    }
}
