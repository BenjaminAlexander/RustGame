use std::thread;
use crate::threading::listener::ListenerTrait;
use crate::threading::listener::sender::Sender;

pub struct JoinHandle<T: ListenerTrait> {
    pub(super) sender: Sender<T>,
    pub(super) join_handle: thread::JoinHandle<T::ThreadReturn>
}

impl<T: ListenerTrait> JoinHandle<T> {

    pub fn get_sender(&self) -> &Sender<T> { &self.sender }

}