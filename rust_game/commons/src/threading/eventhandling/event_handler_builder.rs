use std::io::Error;

use crate::{
    factory::FactoryTrait,
    threading::{
        channel::Receiver,
        eventhandling::{
            event_sender::EventSender,
            EventHandlerTrait,
            EventOrStopThread,
        },
    },
};

pub struct EventHandlerBuilder<T: EventHandlerTrait> {
    sender: EventSender<T::Event>,
    receiver: Receiver<EventOrStopThread<T::Event>>,
}

impl<T: EventHandlerTrait> EventHandlerBuilder<T> {
    pub fn new(factory: &impl FactoryTrait) -> Self {
        let (sender, receiver) = factory.new_channel().take();

        return Self {
            sender: EventSender::new(sender),
            receiver,
        };
    }

    pub fn get_sender(&self) -> &EventSender<T::Event> {
        return &self.sender;
    }

    pub fn spawn_thread(
        self,
        thread_name: String,
        event_handler: T,
        join_call_back: impl FnOnce(T::ThreadReturn) + Send + 'static,
    ) -> Result<EventSender<T::Event>, Error> {
        self.receiver
            .spawn_event_handler(thread_name, event_handler, join_call_back)?;
        return Ok(self.sender);
    }

    pub fn new_thread(
        factory: &impl FactoryTrait,
        thread_name: String,
        event_handler: T,
    ) -> Result<EventSender<T::Event>, Error> {
        return Self::new(factory).spawn_thread(thread_name, event_handler, |_|{});
    }
}
