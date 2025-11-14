use crate::real_time::{EventHandleResult, HandleEvent, ReceiveMetaData, net::tcp::TcpReadHandlerTrait, real::net::tcp::{RealTcpStream, resetable_reader::{DeserializeResult, ResetableReader}}};
use std::ops::ControlFlow;

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

impl<T: TcpReadHandlerTrait> HandleEvent for TcpReaderEventHandler<T> {
    type Event = ();
    type ThreadReturn = ();

    fn on_event(&mut self, _: ReceiveMetaData, _: Self::Event) -> EventHandleResult<Self> {
        return EventHandleResult::TryForNextEvent;
    }

    fn on_channel_empty(&mut self) -> EventHandleResult<Self> {
        return self.read();
    }
    
    fn on_channel_disconnect(&mut self) -> EventHandleResult<Self> {
        return EventHandleResult::StopThread(());
    }

    fn on_timeout(&mut self) -> EventHandleResult<Self> {
        return EventHandleResult::TryForNextEvent;
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn {
        return ();
    }
}
