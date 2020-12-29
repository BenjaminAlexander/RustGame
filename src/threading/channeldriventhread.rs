use crate::threading::{ChannelThread, Receiver};
use log::{info, warn, error};

pub trait ChannelDrivenThread: Send + 'static {

    fn onNonePending(&mut self);

}

impl<T: ChannelDrivenThread> ChannelThread<()> for T {
    fn run(mut self, receiver: Receiver<Self>) -> () {
        info!("Starting");

        while true {
            receiver.recv_try_iter(&mut self).unwrap();
            self.onNonePending();
        }

        info!("Ending");
    }
}