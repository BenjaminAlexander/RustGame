use std::io::Error;
use std::thread::{Builder, JoinHandle};
use log::info;
use crate::threading::channel::ChannelThreadBuilder;
use crate::threading::eventhandling::{EventHandlerTrait, EventOrStopThread};
use crate::threading::{eventhandling, Thread};

pub struct ThreadBuilder {
    name: Option<String>
}

impl ThreadBuilder {

    pub fn new() -> Self {
        return Self {
            name: None
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        return self;
    }

    pub fn build_channel_thread<T: Send + 'static>(self) -> ChannelThreadBuilder<T> {
        return ChannelThreadBuilder::new(self);
    }

    pub fn build_channel_for_event_handler<T: EventHandlerTrait>(self) -> ChannelThreadBuilder<EventOrStopThread<T::Event>> {
        return self.build_channel_thread();
    }

    pub fn spawn_event_handler<T: EventHandlerTrait>(self, event_handler: T) -> std::io::Result<eventhandling::JoinHandle<T>> {
        return self.build_channel_for_event_handler::<T>().spawn_event_handler(event_handler);
    }

    pub(super) fn spawn_thread<T: Thread>(mut self, thread: T) -> std::io::Result<JoinHandle<T::ReturnType>> {
        let mut builder = Builder::new();

        if let Some(name) = self.name.take() {
            builder = builder.name(name);
        }

        return builder.spawn(||{

            info!("Thread Starting");

            let return_value = thread.run();

            info!("Thread Ending");

            return return_value;
        });
    }
}