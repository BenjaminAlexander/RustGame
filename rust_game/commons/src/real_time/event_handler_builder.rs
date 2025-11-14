use std::io::Error;

use crate::real_time::{
    real,
    receiver::ReceiverImplementation,
    simulation,
    EventOrStopThread,
    EventSender,
    FactoryTrait,
    HandleEvent,
    Receiver,
};

pub struct EventHandlerBuilder<T: HandleEvent> {
    sender: EventSender<T::Event>,
    receiver: Receiver<EventOrStopThread<T::Event>>,
}

impl<T: HandleEvent> EventHandlerBuilder<T> {
    pub fn new(factory: &impl FactoryTrait) -> Self {
        let (sender, receiver) = factory.new_channel();

        return Self {
            sender: EventSender::new(sender),
            receiver,
        };
    }

    pub fn get_sender(&self) -> &EventSender<T::Event> {
        return &self.sender;
    }

    pub fn spawn_thread_with_callback(
        self,
        thread_name: String,
        event_handler: T,
        join_call_back: impl FnOnce(T::ThreadReturn) + Send + 'static,
    ) -> Result<EventSender<T::Event>, Error> {
        match self.receiver.take_implementation() {
            ReceiverImplementation::Real(real_receiver) => {
                real::spawn_event_handler(
                    thread_name,
                    real_receiver,
                    event_handler,
                    join_call_back,
                )?;
            }
            ReceiverImplementation::Simulated(single_threaded_receiver) => {
                simulation::spawn_event_handler(
                    thread_name,
                    single_threaded_receiver,
                    event_handler,
                    join_call_back,
                );
            }
        };
        return Ok(self.sender);
    }

    pub fn spawn_thread(
        self,
        thread_name: String,
        event_handler: T,
    ) -> Result<EventSender<T::Event>, Error> {
        return self.spawn_thread_with_callback(thread_name, event_handler, |_| {});
    }

    pub fn new_thread(
        factory: &impl FactoryTrait,
        thread_name: String,
        event_handler: T,
    ) -> Result<EventSender<T::Event>, Error> {
        return Self::new(factory).spawn_thread(thread_name, event_handler);
    }
}
