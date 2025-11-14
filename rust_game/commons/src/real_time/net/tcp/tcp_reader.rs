use std::io::Error;

use crate::real_time::net::tcp::tcp_read_handler_trait::TcpReadHandlerTrait;
use crate::real_time::real::net::tcp::RealTcpStream;
use crate::real_time::simulation::SingleThreadedReceiver;
use crate::real_time::{
    EventOrStopThread,
    Receiver,
};

enum Implementation {
    Real(RealTcpStream),

    //TODO: conditionally compile
    Simulated(SingleThreadedReceiver<Vec<u8>>),
}

//TODO: rename to Reader
pub struct TcpReader {
    implementation: Implementation,
}

impl TcpReader {
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
        thread_name: String,
        receiver: Receiver<EventOrStopThread<()>>,
        tcp_read_handler: T,
        join_call_back: impl FnOnce(()) + Send + 'static,
    ) -> Result<(), Error> {
        match self.implementation {
            Implementation::Real(real_tcp_stream) => real_tcp_stream.spawn_real_tcp_reader(
                thread_name,
                receiver,
                tcp_read_handler,
                join_call_back,
            ),
            Implementation::Simulated(single_threaded_receiver) => single_threaded_receiver
                .spawn_simulated_tcp_reader(
                    thread_name,
                    receiver,
                    tcp_read_handler,
                    join_call_back,
                ),
        }
    }
}
