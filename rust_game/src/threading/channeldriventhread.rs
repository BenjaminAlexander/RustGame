use log::info;

use crate::threading::{ChannelThread, Receiver};

pub trait ChannelDrivenThread: Send + 'static {

    fn on_none_pending(&mut self){
        //No-op
    }

}

impl<T: ChannelDrivenThread> ChannelThread<()> for T {



    fn run(mut self, receiver: Receiver<Self>) -> () {
        info!("Starting");

        loop {
            receiver.recv_try_iter(&mut self).unwrap();
            self.on_none_pending();
        }
    }
}