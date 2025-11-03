use crate::single_threaded_simulator::channel::receiverlink::ReceiverLink;

pub struct SenderLink<T> {
    receiver_link: ReceiverLink<T>,
}

impl<T> SenderLink<T> {
    pub(super) fn new(receiver_link: ReceiverLink<T>) -> Self {
        return Self { receiver_link };
    }

    pub(super) fn send(&self, t: T) -> Result<(), T> {
        return self.receiver_link.send(t);
    }
}

impl<T> Drop for SenderLink<T> {
    fn drop(&mut self) {
        self.receiver_link.disconnect_sender();
    }
}
