use std::thread::Builder;
use log::info;
use crate::threading::channel;
use crate::threading::eventhandling::{EventHandlerTrait, EventOrStopThread};
use crate::threading::{eventhandling, Thread};
use crate::threading::asyncjoin::AsyncJoin;
use crate::threading::listener::{ListenerState, ListenerTrait};

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

    pub fn get_name(&self) -> Option<&String> {
        return self.name.as_ref();
    }

    pub fn build_channel_thread<T: Send + 'static>(self) -> channel::ThreadBuilder<T> {
        return channel::ThreadBuilder::new(self);
    }

    pub fn build_channel_for_event_handler<T: EventHandlerTrait>(self) -> channel::ThreadBuilder<EventOrStopThread<T::Event>> {
        return self.build_channel_thread();
    }

    pub fn spawn_event_handler<T: EventHandlerTrait>(self, event_handler: T, join_call_back: impl FnOnce(AsyncJoin<T::ThreadReturn>) + Send + 'static) -> std::io::Result<eventhandling::Sender<T::Event>> {
        return self.build_channel_for_event_handler::<T>().spawn_event_handler(event_handler, join_call_back);
    }

    pub fn spawn_listener<T: ListenerTrait>(self, listener: T, join_call_back: impl FnOnce(AsyncJoin<T::ThreadReturn>) + Send + 'static) -> std::io::Result<eventhandling::Sender<T::Event>> {
        return self.build_channel_for_event_handler::<ListenerState<T>>().spawn_listener(listener, join_call_back);
    }

    pub(super) fn spawn_thread<T: Thread>(self, thread: T, join_call_back: impl FnOnce(AsyncJoin<T::ReturnType>) + Send + 'static) -> std::io::Result<()> {
        let mut builder = Builder::new();

        if let Some(name) = self.name.as_ref() {
            builder = builder.name(name.clone());
        }

        builder.spawn(||{

            info!("Thread Starting");

            let return_value = thread.run();
            let async_join = AsyncJoin::new(self, return_value);
            join_call_back(async_join);

            info!("Thread Ending");
        })?;

        return Ok(());
    }
}