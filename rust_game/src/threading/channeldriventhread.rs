use log::{trace, info};

use crate::threading::{ChannelThread, Receiver};
use std::sync::mpsc::{TryRecvError};
use crate::threading::channeldriventhread::ThreadAction::{Continue, Stop};

pub enum ThreadAction<T> {
    Continue,
    Stop(T)
}

pub trait ChannelDrivenThread<T>: Send + 'static
    where T: Send + 'static {

    fn after_message(&mut self) -> ThreadAction<T> {
        Continue
    }

    fn on_none_pending(&mut self) -> ThreadAction<T> {
        Continue
    }

    fn on_channel_disconnect(&mut self) -> T;
}

impl<T, U: ChannelDrivenThread<T>> ChannelThread<T> for U
    where T: Send + 'static {

    fn run(mut self, receiver: Receiver<Self>) -> T {
        info!("Starting");

        loop {
            trace!("Waiting.");
            match receiver.recv(&mut self) {
                Err(_error) => {
                    return self.on_channel_disconnect();
                }
                _ => {}
            }

            match self.after_message() {
                Stop(return_value) => {
                    info!("Thread is returning.");
                    return return_value;
                }
                Continue => {
                    loop {
                        trace!("Looking for more.");
                        match receiver.try_recv(&mut self) {
                            Ok(()) => {
                                match self.after_message() {
                                    Stop(return_value) => {
                                        info!("Thread is returning.");
                                        return return_value;
                                    }
                                    Continue => {}
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
            }

            trace!("None left.");
            match self.on_none_pending() {
                Stop(return_value) => {
                    info!("Thread is returning.");
                    return return_value;
                }
                Continue => {}
            }
        }
    }
}