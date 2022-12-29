use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, SendError, TryRecvError};

pub type MessageChannelTryRecvError = TryRecvError;

pub type MessageChannelSendError<T> = SendError<MessageHolder<T>>;

pub fn message_channel<T: Send + 'static>() -> (MessageChannelSender<T>, MessageChannelReceiver<T>) {

    let (sender, receiver): (Sender<MessageHolder<T>>, Receiver<MessageHolder<T>>) = mpsc::channel();

    return (
        MessageChannelSender{sender},
        MessageChannelReceiver{receiver}
    );
}

pub struct MessageHolder<T> {
    message: T
}

pub struct MessageChannelSender<T> {
    sender: Sender<MessageHolder<T>>
}

pub struct MessageChannelReceiver<T> {
    receiver: Receiver<MessageHolder<T>>
}

impl<T> Clone for MessageChannelSender<T> {
    fn clone(&self) -> Self {
        Self {sender: self.sender.clone()}
    }
}

impl<T> MessageChannelSender<T> {

    pub fn send(&self, message: T) -> Result<(), MessageChannelSendError<T>> {
        return self.sender.send(MessageHolder::<T>{message});
    }

}

impl<T> MessageChannelReceiver<T> {

    pub fn try_recv(&self) -> Result<T, MessageChannelTryRecvError> {
        return match self.try_recv_holder() {
            Ok(message_holder) => Ok(message_holder.message),
            Err(error) => Err(error)
        }
    }

    pub fn try_recv_holder(&self) -> Result<MessageHolder<T>, MessageChannelTryRecvError> {
        return self.receiver.try_recv();
    }
}