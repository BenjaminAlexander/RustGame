use std::io::Error;

use crate::net::{
    RealTcpStream,
    TcpReadHandlerTrait,
};
use crate::single_threaded_simulator::SingleThreadedReceiver;
use crate::threading::channel::Receiver;
use crate::threading::eventhandling::EventOrStopThread;
use crate::threading::{
    AsyncJoinCallBackTrait,
    ThreadBuilder,
};

enum Implementation {
    Real(RealTcpStream),

    //TODO: conditionally compile
    Simulated(SingleThreadedReceiver<Vec<u8>>),
}

//TODO: rename to Reader
pub struct TcpReceiver {
    implementation: Implementation,
}

impl TcpReceiver {
    pub fn new(real_tcp_stream: RealTcpStream) -> Self {
        return Self {
            implementation: Implementation::Real(real_tcp_stream),
        };
    }

    pub fn new_simulated(channel_tcp_writer: SingleThreadedReceiver<Vec<u8>>) -> Self {
        return Self {
            implementation: Implementation::Simulated(channel_tcp_writer),
        };
    }

    pub fn spawn_tcp_reader<T: TcpReadHandlerTrait>(
        self,
        thread_builder: ThreadBuilder,
        receiver: Receiver<EventOrStopThread<()>>,
        tcp_read_handler: T,
        join_call_back: impl AsyncJoinCallBackTrait<T>,
    ) -> Result<(), Error> {
        match self.implementation {
            Implementation::Real(real_tcp_stream) => real_tcp_stream.spawn_real_tcp_reader(
                thread_builder,
                receiver,
                tcp_read_handler,
                join_call_back,
            ),
            Implementation::Simulated(single_threaded_receiver) => single_threaded_receiver
                .spawn_simulated_tcp_reader(
                    thread_builder,
                    receiver,
                    tcp_read_handler,
                    join_call_back,
                ),
        }
    }
}
