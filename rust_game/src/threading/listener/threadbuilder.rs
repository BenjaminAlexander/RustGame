use std::io::Error;
use crate::{ThreadBuilderTrait, threading};
use crate::threading::eventhandling::Thread;
use crate::threading::listener::eventhandler::ListenerState;
use crate::threading::listener::ListenerTrait;
use crate::threading::listener::joinhandle::JoinHandle;
use crate::threading::listener::sender::Sender;

pub struct ThreadBuilder<T: ListenerTrait> {
    pub(super) sender: Sender<T>,
    pub(super) builder: threading::ThreadBuilder<Thread<ListenerState<T>>>
}

impl<T: ListenerTrait> ThreadBuilder<T> {

    pub fn get_sender(&self) -> &Sender<T> { &self.sender }

}

impl<T: ListenerTrait> ThreadBuilderTrait for ThreadBuilder<T> {
    type StartResultType = Result<JoinHandle<T>, Error>;

    fn name(mut self, name: &str) -> Self {
        self.builder = self.builder.name(name);
        return self;
    }

    fn start(self) -> Self::StartResultType {
        let join_handle = self.builder.start()?;

        return Result::Ok(JoinHandle {
            sender: self.sender,
            join_handle
        });
    }
}