use std::sync::mpsc::{Sender as MpscSender, Receiver as MpscReceiver};
use crate::threading::{Sender, Receiver};
use std::sync::mpsc;

pub fn channel<T: Send + 'static, U>() -> (Sender<T, U>, Receiver<T, U>) {
    let (sender, receiver): (MpscSender<Box<dyn FnOnce(&mut T) -> U + Send + 'static>>, MpscReceiver<Box<dyn FnOnce(&mut T) -> U + Send + 'static>>) = mpsc::channel();
    (Sender::<T, U>::new(sender), Receiver::<T, U>::new(receiver))
}