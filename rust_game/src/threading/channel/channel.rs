use std::sync::mpsc;
use crate::threading::channel::{SentValueHolder, Receiver, Sender};

pub fn message_channel<T: Send + 'static>() -> (Sender<T>, Receiver<T>) {

    let (sender, receiver): (mpsc::Sender<SentValueHolder<T>>, mpsc::Receiver<SentValueHolder<T>>) = mpsc::channel();

    return (
        Sender { sender },
        Receiver { receiver }
    );
}