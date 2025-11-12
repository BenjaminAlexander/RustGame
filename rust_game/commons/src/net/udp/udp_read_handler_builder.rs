use std::io::Error;

use crate::{
    factory::FactoryTrait,
    net::{
        UdpReadHandlerTrait,
        UdpSocket,
    },
    threading::{
        channel::Receiver,
        eventhandling::{
            EventHandlerStopper,
            EventOrStopThread,
            EventSender,
        },
    },
};

pub struct UdpReadHandlerBuilder {
    stopper: EventHandlerStopper,
    receiver: Receiver<EventOrStopThread<()>>,
}

impl UdpReadHandlerBuilder {
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

    pub fn spawn_thread_with_call_back<T: UdpReadHandlerTrait>(
        self,
        thread_name: String,
        udp_socket: UdpSocket,
        udp_read_handler: T,
        join_call_back: impl FnOnce(()) + Send + 'static,
    ) -> Result<EventHandlerStopper, Error> {
        udp_socket.spawn_udp_reader(
            thread_name,
            self.receiver,
            udp_read_handler,
            join_call_back,
        )?;
        return Ok(self.stopper);
    }

    pub fn spawn_thread<T: UdpReadHandlerTrait>(
        self,
        thread_name: String,
        udp_socket: UdpSocket,
        udp_read_handler: T,
    ) -> Result<EventHandlerStopper, Error> {
        udp_socket.spawn_udp_reader(thread_name, self.receiver, udp_read_handler, |_| {})?;
        return Ok(self.stopper);
    }

    pub fn new_thread<T: UdpReadHandlerTrait>(
        factory: &impl FactoryTrait,
        thread_name: String,
        udp_socket: UdpSocket,
        udp_read_handler: T,
    ) -> Result<EventHandlerStopper, Error> {
        return Self::new(factory).spawn_thread(thread_name, udp_socket, udp_read_handler);
    }
}
