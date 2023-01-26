use std::thread;
use crate::threading::eventhandling::eventhandlertrait::EventHandlerTrait;
use crate::threading::eventhandling::Sender;

//TODO: don't reference EventHandlerTrait
//TODO: move this to channel
//TODO: make event handling type alias
pub struct JoinHandle<T: EventHandlerTrait> {
    pub(crate) sender: Sender<T::Event>,
    pub(crate) join_handle: thread::JoinHandle<T::ThreadReturn>
}

impl<T: EventHandlerTrait> JoinHandle<T> {

    pub fn get_sender(&self) -> &Sender<T::Event> { &self.sender }

}