use std::io::Error;

use crate::{
    net::{
        TcpReadHandlerTrait,
        TcpReader,
    }, real_time::{EventHandlerStopper, EventSender, FactoryTrait, Receiver}, threading::eventhandling::EventOrStopThread
};

pub struct TcpReadHandlerBuilder {
    stopper: EventHandlerStopper,
    receiver: Receiver<EventOrStopThread<()>>,
}

impl TcpReadHandlerBuilder {
    pub fn new(factory: &impl FactoryTrait) -> Self {
        let (sender, receiver) = factory.new_channel();

        return Self {
            stopper: EventHandlerStopper::new(EventSender::new(sender)),
            receiver,
        };
    }

    pub fn get_stopper(&self) -> &EventHandlerStopper {
        return &self.stopper;
    }

    pub fn spawn_thread_with_call_back<T: TcpReadHandlerTrait>(
        self,
        thread_name: String,
        tcp_reader: TcpReader,
        tcp_read_handler: T,
        join_call_back: impl FnOnce(()) + Send + 'static,
    ) -> Result<EventHandlerStopper, Error> {
        tcp_reader.spawn_tcp_reader(
            thread_name,
            self.receiver,
            tcp_read_handler,
            join_call_back,
        )?;
        return Ok(self.stopper);
    }

    pub fn spawn_thread<T: TcpReadHandlerTrait>(
        self,
        thread_name: String,
        tcp_reader: TcpReader,
        tcp_read_handler: T,
    ) -> Result<EventHandlerStopper, Error> {
        tcp_reader.spawn_tcp_reader(thread_name, self.receiver, tcp_read_handler, |_| {})?;
        return Ok(self.stopper);
    }

    pub fn new_thread<T: TcpReadHandlerTrait>(
        factory: &impl FactoryTrait,
        thread_name: String,
        tcp_reader: TcpReader,
        tcp_read_handler: T,
    ) -> Result<EventHandlerStopper, Error> {
        return Self::new(factory).spawn_thread(thread_name, tcp_reader, tcp_read_handler);
    }
}
