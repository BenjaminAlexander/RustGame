use crate::real_time::real::net::tcp::RealTcpStream;
use crate::real_time::simulation::SingleThreadedReceiver;

pub(super) enum TcpReaderImplementation {
    Real(RealTcpStream),

    //TODO: conditionally compile
    Simulated(SingleThreadedReceiver<Vec<u8>>),
}

//TODO: rename to Reader
pub struct TcpReader {
    implementation: TcpReaderImplementation,
}

impl TcpReader {
    pub fn new(real_tcp_stream: RealTcpStream) -> Self {
        return Self {
            implementation: TcpReaderImplementation::Real(real_tcp_stream),
        };
    }

    pub fn new_simulated(channel_tcp_writer: SingleThreadedReceiver<Vec<u8>>) -> Self {
        return Self {
            implementation: TcpReaderImplementation::Simulated(channel_tcp_writer),
        };
    }

    pub(super) fn take_implementation(self) -> TcpReaderImplementation {
        return self.implementation;
    }
}
