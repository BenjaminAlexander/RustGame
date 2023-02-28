use std::thread;
use crate::threading::channel::Sender;

//TODO: not really a join handle anymore
pub struct JoinHandle<T> {
    pub(crate) sender: Sender<T>,
    pub(crate) join_handle: thread::JoinHandle<()>
}

impl<T> JoinHandle<T> {
    pub fn get_sender(&self) -> &Sender<T> { &self.sender }
}