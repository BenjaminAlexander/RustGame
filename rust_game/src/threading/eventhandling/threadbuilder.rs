use crate::{OldThreadBuilderTrait, threading};
use crate::threading::channel::{ChannelThreadBuilder, ChannelThreadBuilderBuilder};
use crate::threading::eventhandling::eventhandlertrait::EventHandlerTrait;
use crate::threading::eventhandling::thread::Thread;
use crate::threading::eventhandling::{EventOrStopThread, Sender};
use crate::threading::eventhandling::joinhandle::JoinHandle;

pub struct ThreadBuilder<T: EventHandlerTrait> {
    pub(in crate::threading) sender: Sender<T::Event>,
    pub(in crate::threading) builder: threading::OldThreadBuilder<Thread<T>>
}

impl<T: EventHandlerTrait> ThreadBuilder<T> {

    pub fn get_sender(&self) -> &Sender<T::Event> { &self.sender }

}

impl<T: EventHandlerTrait> OldThreadBuilderTrait for ThreadBuilder<T> {
    type StartResultType = std::io::Result<JoinHandle<T>>;

    fn name(mut self, name: &str) -> Self {
        self.builder = self.builder.name(name);
        return self;
    }

    fn start(self) -> std::io::Result<JoinHandle<T>> {
        let join_handle = self.builder.start()?;

        return Result::Ok(JoinHandle {
            sender: self.sender,
            join_handle
        });
    }
}

pub trait EventHandlerThreadBuilderTrait {

    fn build_channel_for_event_handler<T: EventHandlerTrait>(self) -> ChannelThreadBuilder<EventOrStopThread<T::Event>>;

    fn spawn_event_handler<T: EventHandlerTrait>(self, event_handler: T) -> std::io::Result<JoinHandle<T>>;

}

impl EventHandlerThreadBuilderTrait for threading::ThreadBuilder {

    fn build_channel_for_event_handler<T: EventHandlerTrait>(self) -> ChannelThreadBuilder<EventOrStopThread<T::Event>> {
        return self.build_channel_thread();
    }

    fn spawn_event_handler<T: EventHandlerTrait>(self, event_handler: T) -> std::io::Result<JoinHandle<T>> {
        return self.build_channel_for_event_handler::<T>().spawn_event_handler(event_handler);
    }
}

pub trait EventHandlerChannelThreadBuilderTrait<T: EventHandlerTrait> {

    fn spawn_event_handler(self, event_handler: T) -> std::io::Result<JoinHandle<T>>;

}

impl<T: EventHandlerTrait> EventHandlerChannelThreadBuilderTrait<T> for ChannelThreadBuilder<EventOrStopThread<T::Event>> {

    fn spawn_event_handler(self, event_handler: T) -> std::io::Result<JoinHandle<T>> {

        let (thread_builder, channel) = self.take();

        let (sender, receiver) = channel.take();

        let thread = Thread {
            receiver,
            event_handler
        };

        let join_handle = thread_builder.spawn_thread(thread)?;

        return Result::Ok(JoinHandle {
            sender,
            join_handle
        });
    }

}