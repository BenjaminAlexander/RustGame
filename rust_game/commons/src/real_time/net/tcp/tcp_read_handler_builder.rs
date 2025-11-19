use std::io::Error;

use crate::real_time::{
    event_or_stop_thread::EventOrStopThread,
    net::tcp::{
        handle_tcp_read::HandleTcpRead,
        tcp_reader::TcpReaderImplementation,
        TcpReader,
    },
    real::net::tcp::RealTcpReaderEventHandler,
    receiver::ReceiverImplementation,
    simulation::net::tcp::SimulatedTcpReaderEventHandler,
    EventHandlerStopper,
    EventSender,
    Factory,
    Receiver,
};

pub struct TcpReadHandlerBuilder {
    stopper: EventHandlerStopper,
    receiver: Receiver<EventOrStopThread<()>>,
}

impl TcpReadHandlerBuilder {
    pub fn new(factory: &Factory) -> Self {
        let (sender, receiver) = factory.new_channel();

        return Self {
            stopper: EventHandlerStopper::new(EventSender::new(sender)),
            receiver,
        };
    }

    pub fn get_stopper(&self) -> &EventHandlerStopper {
        return &self.stopper;
    }

    pub fn spawn_thread_with_call_back<T: HandleTcpRead>(
        self,
        thread_name: String,
        tcp_reader: TcpReader,
        tcp_read_handler: T,
        join_call_back: impl FnOnce(()) + Send + 'static,
    ) -> Result<EventHandlerStopper, Error> {
        match (self.receiver.take_implementation(), tcp_reader.take_implementation()) {
            (ReceiverImplementation::Real(real_receiver), TcpReaderImplementation::Real(real_tcp_stream)) => RealTcpReaderEventHandler::spawn_tcp_reader(thread_name, real_receiver, real_tcp_stream, tcp_read_handler, join_call_back),
            (ReceiverImplementation::Real(_), TcpReaderImplementation::Simulated(_)) => panic!("Spawning a TCP reader thread with a simulated TCP reader and a real channel isn't supported"),
            (ReceiverImplementation::Simulated(_), TcpReaderImplementation::Real(_)) => panic!("Spawning a TCP reader thread with a real TCP stream and a simulated channel isn't supported"),
            (ReceiverImplementation::Simulated(single_threaded_receiver), TcpReaderImplementation::Simulated(simulated_tcp_stream)) => SimulatedTcpReaderEventHandler::spawn_tcp_reader(thread_name, single_threaded_receiver, simulated_tcp_stream, tcp_read_handler, join_call_back),
        }?;

        return Ok(self.stopper);
    }

    pub fn spawn_thread<T: HandleTcpRead>(
        self,
        thread_name: String,
        tcp_reader: TcpReader,
        tcp_read_handler: T,
    ) -> Result<EventHandlerStopper, Error> {
        return self.spawn_thread_with_call_back(thread_name, tcp_reader, tcp_read_handler, |_| {});
    }

    pub fn new_thread<T: HandleTcpRead>(
        factory: &Factory,
        thread_name: String,
        tcp_reader: TcpReader,
        tcp_read_handler: T,
    ) -> Result<EventHandlerStopper, Error> {
        return Self::new(factory).spawn_thread(thread_name, tcp_reader, tcp_read_handler);
    }
}
