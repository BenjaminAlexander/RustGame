use log::{trace, info};

use crate::threading::{ChannelThread, OldReceiver, OldSender, OldSendError};
use std::sync::mpsc::{TryRecvError};
use crate::threading::channeldriventhread::ThreadAction::{Continue, Stop};

pub enum ThreadAction {
    Continue,
    Stop
}

pub type ChannelDrivenThreadSender<T> = OldSender<T, ThreadAction>;

pub type ChannelDrivenThreadReceiver<T> = OldReceiver<T, ThreadAction>;

pub type ChannelDrivenThreadSenderError<T> = OldSendError<T, ThreadAction>;

pub trait ChannelDrivenThread<T>: Send + 'static
    where T: Send + 'static {

    fn after_message(&mut self) -> ThreadAction {
        Continue
    }

    fn on_none_pending(&mut self) -> ThreadAction {
        Continue
    }

    fn on_channel_disconnect(&mut self) -> T;
}

impl<T, U: ChannelDrivenThread<T>> ChannelThread<T, ThreadAction> for U
    where T: Send + 'static {

    fn run(mut self, receiver: OldReceiver<Self, ThreadAction>) -> T {
        info!("Starting");

        loop {
            trace!("Waiting.");
            match receiver.recv(&mut self) {
                Err(error) => {
                    info!("Thread stopped due to disconnect: {:?}", error);
                    return self.on_channel_disconnect();
                }
                Ok(Stop) => {
                    info!("Thread commanded to stop.");
                    return self.on_channel_disconnect();
                }
                Ok(Continue) => {}
            }

            match self.after_message() {
                Stop => {
                    info!("Thread commanded to stop.");
                    return self.on_channel_disconnect();
                }
                Continue => {
                    loop {
                        trace!("Looking for more.");
                        match receiver.try_recv(&mut self) {
                            Ok(Stop) => {
                                info!("Thread commanded to stop.");
                                return self.on_channel_disconnect();
                            }
                            Ok(Continue) => {
                                match self.after_message() {
                                    Stop => {
                                        info!("Thread commanded to stop.");
                                        return self.on_channel_disconnect();
                                    }
                                    Continue => {}
                                }
                            }
                            Err(TryRecvError::Empty) => break,
                            Err(TryRecvError::Disconnected) => {
                                info!("Thread stopped due to disconnect.");
                                return self.on_channel_disconnect();
                            }
                        }
                    }
                }
            }

            trace!("None left.");
            match self.on_none_pending() {
                Stop => {
                    info!("Thread commanded to stop.");
                    return self.on_channel_disconnect();
                }
                Continue => {}
            }
        }
    }
}