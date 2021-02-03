use std::sync::mpsc::{Sender as MpscSender, Receiver as MpscReceiver};
use crate::threading::{Sender, Receiver};
use std::sync::mpsc;

pub fn channel<T: Send + 'static>() -> (Sender<T>, Receiver<T>) {
    let (sender, receiver): (MpscSender<Box<dyn FnOnce(&mut T) + Send + 'static>>, MpscReceiver<Box<dyn FnOnce(&mut T) + Send + 'static>>) = mpsc::channel();
    (Sender::<T>::new(sender), Receiver::<T>::new(receiver))
}