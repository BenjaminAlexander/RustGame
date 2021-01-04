use log::{trace, info};

use crate::threading::{ChannelThread, Receiver};
use std::sync::mpsc::{TryRecvError};

pub trait ChannelDrivenThread<T>: Send + 'static
    where T: Send + 'static {

    fn after_message(&mut self) -> Option<T> {
        None
    }

    fn on_none_pending(&mut self) -> Option<T> {
        None
    }

    fn on_channel_disconnect(&mut self) -> T;
}

impl<T, U: ChannelDrivenThread<T>> ChannelThread<T> for U
    where T: Send + 'static {

    fn run(mut self, receiver: Receiver<Self>) -> T {
        trace!("Starting");

        loop {
            trace!("Waiting.");
            match receiver.recv(&mut self) {
                Err(_error) => {
                    return self.on_channel_disconnect();
                }
                _ => {}
            }

            match self.after_message() {
                None => {
                    loop {
                        trace!("Looking for more.");
                        match receiver.try_recv(&mut self) {
                            Ok(()) => {
                                match self.after_message() {
                                    Some(return_value) => {
                                        info!("Thread is returning.");
                                        return return_value;
                                    }
                                    _=>{}
                                }
                            }
                            Err(error) => {
                                match error {
                                    TryRecvError::Empty => break,
                                    TryRecvError::Disconnected => return self.on_channel_disconnect()
                                }
                            }
                        }
                    }
                }
                Some(return_value) => {
                    info!("Thread is returning.");
                    return return_value;
                }
            }

            trace!("None left.");
            match self.on_none_pending() {
                None => {/*continue*/}
                Some(return_value) => {
                    info!("Thread is returning.");
                    return return_value;
                }
            }
        }
    }
}