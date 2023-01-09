use std::sync::mpsc::{Sender as MpscSender, Receiver as MpscReceiver};
use crate::threading::{OldSender, OldReceiver};
use std::sync::mpsc;

pub fn old_channel<T: Send + 'static, U>() -> (OldSender<T, U>, OldReceiver<T, U>) {
    let (sender, receiver): (MpscSender<Box<dyn FnOnce(&mut T) -> U + Send + 'static>>, MpscReceiver<Box<dyn FnOnce(&mut T) -> U + Send + 'static>>) = mpsc::channel();
    (OldSender::<T, U>::new(sender), OldReceiver::<T, U>::new(receiver))
}