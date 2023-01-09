use std::sync::mpsc;

//TODO: rename to Sender Receiver once the old ones are gone

pub type TryRecvError = mpsc::TryRecvError;

pub type SendError<T> = mpsc::SendError<SentValueHolder<T>>;

pub type RecvError = mpsc::RecvError;

pub fn message_channel<T: Send + 'static>() -> (ValueSender<T>, ValueReceiver<T>) {

    let (sender, receiver): (mpsc::Sender<SentValueHolder<T>>, mpsc::Receiver<SentValueHolder<T>>) = mpsc::channel();

    return (
        ValueSender {sender},
        ValueReceiver {receiver}
    );
}

pub struct SentValueHolder<T> {
    value: T
}

pub struct ReceivedValueHolder<T> {
    sent_value_holder: SentValueHolder<T>
}

impl<T> ReceivedValueHolder<T> {

    pub fn get_message(&self) -> &T { &self.sent_value_holder.value }

    pub fn move_message(self) -> T { self.sent_value_holder.value }
}

pub struct ValueSender<T> {
    sender: mpsc::Sender<SentValueHolder<T>>
}

pub struct ValueReceiver<T> {
    receiver: mpsc::Receiver<SentValueHolder<T>>
}

impl<T> Clone for ValueSender<T> {
    fn clone(&self) -> Self {
        Self {sender: self.sender.clone()}
    }
}

impl<T> ValueSender<T> {

    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        return self.sender.send(SentValueHolder{value});
    }

}

impl<T> ValueReceiver<T> {

    pub fn try_recv_holder(&self) -> Result<ReceivedValueHolder<T>, TryRecvError> {
        return Ok(ReceivedValueHolder {
            sent_value_holder: self.receiver.try_recv()?
        });
    }

    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        return Ok(self.try_recv_holder()?.move_message());
    }

    pub fn recv_holder(&self) -> Result<ReceivedValueHolder<T>, RecvError> {
        return Ok(ReceivedValueHolder {
            sent_value_holder: self.receiver.recv()?
        });
    }

    pub fn recv(&self) -> Result<T, RecvError> {
        return Ok(self.recv_holder()?.move_message());
    }
}