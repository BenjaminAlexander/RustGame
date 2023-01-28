use crate::{OldThreadBuilderTrait, threading};
use crate::threading::channel::ChannelThreadBuilder;
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
    type StartResultType = std::io::Result<JoinHandle<T::Event, T::ThreadReturn>>;

    fn name(mut self, name: &str) -> Self {
        self.builder = self.builder.name(name);
        return self;
    }

    fn start(self) -> std::io::Result<JoinHandle<T::Event, T::ThreadReturn>> {
        let join_handle = self.builder.start()?;

        return Result::Ok(JoinHandle {
            sender: self.sender,
            join_handle
        });
    }
}