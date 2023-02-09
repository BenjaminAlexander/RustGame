use std::thread;
use crate::threading::channel::Sender;

pub struct JoinHandle<T, U> {
    pub(crate) sender: Sender<T>,
    pub(crate) join_handle: thread::JoinHandle<U>
}

impl<T, U> JoinHandle<T, U> {

    pub fn get_sender(&self) -> &Sender<T> { &self.sender }

    pub fn join(self) -> std::thread::Result<U> {
        return self.join_handle.join();
    }
}