use std::sync::mpsc;

pub type SendError<T> = mpsc::SendError<SentValueHolder<T>>;

//TODO: switch to sent value meta data struct
pub struct SentValueHolder<T> {
    pub(super) value: T
}

pub struct Sender<T> {
    pub(super) sender: mpsc::Sender<SentValueHolder<T>>
}

impl<T> Sender<T> {

    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        return self.sender.send(SentValueHolder { value } );
    }

}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self { sender: self.sender.clone() }
    }
}
