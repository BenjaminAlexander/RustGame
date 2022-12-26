use std::sync::mpsc::{Receiver as MpscReceiver, RecvError, TryRecvError, RecvTimeoutError};
use core::time::Duration;

pub struct Receiver<T: ?Sized, U> {
    receiver: MpscReceiver<Box<dyn FnOnce(&mut T) -> U + Send + 'static>>
}

impl<T, U> Receiver<T, U> {
    pub fn new(receiver: MpscReceiver<Box<dyn FnOnce(&mut T) -> U + Send + 'static>>) -> Self {
        Receiver { receiver }
    }

    pub fn recv(&self, t: &mut T) -> Result<U, RecvError> {
        let message = self.receiver.recv()?;
        return Self::apply_message(message, t);
    }

    pub fn recv_timeout(&self, t: &mut T, timeout: Duration) -> Result<U, RecvTimeoutError> {
        let message = self.receiver.recv_timeout(timeout)?;
        return Self::apply_message(message, t);
    }

    pub fn try_recv(&self, t: &mut T) -> Result<U, TryRecvError> {
        let message = self.receiver.try_recv()?;
        return Self::apply_message(message, t);
    }

    // pub fn iter(&self, t: &mut T) {
    //     for message in self.receiver.iter() {
    //         message(t);
    //     }
    // }

    //pub fn try_iter(&self, t: &mut T) {
    //    for message in self.receiver.try_iter() {
    //        message(t);
    //    }
    //}

    //pub fn recv_try_iter(&self, t: &mut T) -> Result<(), RecvError> {
    //    self.recv(t)?;
    //    self.try_iter(t);
    //    Ok(())
    //}

    fn apply_message<V>(message: Box<dyn FnOnce(&mut T) -> U + Send + 'static>, t: &mut T) -> Result<U, V> {
        Ok(message(t))
    }

    // pub fn bundle(self, t: T) -> ReceiverBundle<T> {
    //     ReceiverBundle::new(t, self)
    // }
}

// pub struct ReceiverBundle<T> {
//     receiver: Receiver<T>,
//     val: T
// }
//
// impl<T> ReceiverBundle<T> {
//     fn new(val: T, receiver: Receiver<T>) -> Self {
//         ReceiverBundle{val, receiver}
//     }
//
//     pub fn recv(&mut self) -> Result<(), RecvError> {
//         self.receiver.recv(&mut self.val)?;
//         Ok(())
//     }
//
//     pub fn try_recv(&mut self) -> Result<(), TryRecvError> {
//         self.receiver.try_recv(&mut self.val)?;
//         Ok(())
//     }
//
//     pub fn iter(&mut self) {
//         self.receiver.iter(&mut self.val);
//     }
//
//     pub fn try_iter(&mut self) {
//         self.receiver.try_iter(&mut self.val);
//     }
//
//     pub fn recv_try_iter(&mut self) -> Result<(), RecvError> {
//         self.receiver.recv_try_iter(&mut self.val)?;
//         Ok(())
//     }
//
//     pub fn get_receiver(&self) -> &Receiver<T> {
//         & self.receiver
//     }
//
//     pub fn get(&mut self) -> &mut T {
//         &mut self.val
//     }
// }