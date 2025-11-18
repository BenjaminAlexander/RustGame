use commons::real_time::{
    net::tcp::TcpStream,
    EventHandleResult,
    HandleEvent,
    ReceiveMetaData,
};

//TODO: Send response to time messages to calculate ping
pub struct TcpOutput {
    tcp_stream: TcpStream,
}

impl TcpOutput {
    pub fn new(tcp_stream: TcpStream) -> Self {
        return Self { tcp_stream };
    }
}

impl HandleEvent for TcpOutput {
    type Event = ();
    type ThreadReturn = ();

    fn on_event(&mut self, _: ReceiveMetaData, _: Self::Event) -> EventHandleResult {
        EventHandleResult::TryForNextEvent
    }
    
    fn on_stop_self(self) -> Self::ThreadReturn {
        ()
    }
}
