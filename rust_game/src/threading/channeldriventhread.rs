use log::{trace};

use crate::threading::{ChannelThread, Receiver};
use std::sync::mpsc::TryRecvError;

pub trait ChannelDrivenThread<T>: Send + 'static
    where T: Send + 'static {

    fn after_message(&mut self) -> Option<T> {
        None
    }

    fn on_none_pending(&mut self)-> Option<T> {
        None
    }

}

impl<T, U: ChannelDrivenThread<T>> ChannelThread<Result<T, TryRecvError>> for U
    where T: Send + 'static {

    fn run(mut self, receiver: Receiver<Self>) -> Result<T, TryRecvError> {
        trace!("Starting");

        loop {
            trace!("Waiting.");
            receiver.recv(&mut self).unwrap();

            match self.after_message() {
                None => {
                    loop {
                        trace!("Looking for more.");
                        match receiver.try_recv(&mut self) {
                            Ok(()) => {
                                match self.after_message() {
                                    None => {
                                        /*Continue*/
                                    }
                                    Some(return_value) => {
                                        trace!("Returning.");
                                        return Ok(return_value);
                                    }
                                }
                            }
                            Err(error) => {
                                match error {
                                    TryRecvError::Empty => break,
                                    TryRecvError::Disconnected => return Err(error)
                                }
                            }
                        }
                    }
                }
                Some(return_value) => {
                    trace!("Returning.");
                    return Ok(return_value);
                }
            }

            trace!("None left.");
            match self.on_none_pending() {
                None => {/*continue*/}
                Some(return_value) => {
                    trace!("Returning.");
                    return Ok(return_value);}
            }
        }
    }
}