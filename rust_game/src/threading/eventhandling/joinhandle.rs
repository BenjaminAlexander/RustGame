use std::thread::JoinHandle as BaseJoinHandle;
use crate::threading::eventhandling::eventhandler::EventHandlerTrait;
use crate::threading::eventhandling::EventSender;

pub struct JoinHandle<T: EventHandlerTrait> {
    pub(super) sender: EventSender<T>,
    pub(super) join_handle: BaseJoinHandle<T::ThreadReturnType>
}

impl<T: EventHandlerTrait> JoinHandle<T> {

    pub fn get_sender(&self) -> &EventSender<T> { &self.sender }

}