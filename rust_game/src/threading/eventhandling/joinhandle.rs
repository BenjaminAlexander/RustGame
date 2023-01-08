use std::thread;
use crate::threading::eventhandling::eventhandlertrait::EventHandlerTrait;
use crate::threading::eventhandling::Sender;

pub struct JoinHandle<T: EventHandlerTrait> {
    pub(super) sender: Sender<T>,
    pub(super) join_handle: thread::JoinHandle<T::ThreadReturn>
}

impl<T: EventHandlerTrait> JoinHandle<T> {

    pub fn get_sender(&self) -> &Sender<T> { &self.sender }

}