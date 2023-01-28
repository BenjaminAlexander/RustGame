use std::thread;
use crate::threading::eventhandling::Sender;

//TODO: don't reference EventHandlerTrait
//TODO: move this to channel
//TODO: make event handling type alias
pub struct JoinHandle<T, U> {
    pub(crate) sender: Sender<T>,
    pub(crate) join_handle: thread::JoinHandle<U>
}

impl<T, U> JoinHandle<T, U> {

    pub fn get_sender(&self) -> &Sender<T> { &self.sender }

}