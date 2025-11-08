use std::io::Error;

use crate::{factory::FactoryTrait, threading::{AsyncJoinCallBackTrait, channel::Receiver, eventhandling::{EventHandlerTrait, EventOrStopThread, event_sender::EventSender}}};

pub struct EventHandlerBuilder<T: EventHandlerTrait> {
    sender: EventSender<T::Event>,
    receiver: Receiver<EventOrStopThread<T::Event>>
}

impl<T: EventHandlerTrait> EventHandlerBuilder<T> {
    pub fn new(factor: &impl FactoryTrait) -> Self {

        let (sender, receiver) = factor.new_channel().take();

        return Self {
            sender: EventSender::new(sender),
            receiver
        }
    }

    pub fn get_sender(&self) -> &EventSender<T::Event> {
        return &self.sender;
    }

    pub fn spawn_thread(
        self,
        thread_name: String,
        event_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<T::ThreadReturn>,
    ) -> Result<EventSender<T::Event>, Error> {
        self.receiver.spawn_event_handler(thread_name, event_handler, join_call_back)?;
        return Ok(self.sender);
    }
}