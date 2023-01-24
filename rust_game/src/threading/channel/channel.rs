use std::sync::mpsc;
use crate::threading::channel::{Receiver, Sender, SendMetaData};

pub fn message_channel<T: Send + 'static>() -> (Sender<T>, Receiver<T>) {

    let (sender, receiver): (mpsc::Sender<(SendMetaData, T)>, mpsc::Receiver<(SendMetaData, T)>) = mpsc::channel();

    return (
        Sender { sender },
        Receiver { receiver }
    );
}