use commons::net::TcpStream;
use commons::real_time::{EventHandleResult, HandleEvent, ReceiveMetaData};

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

    fn on_stop(self, _: ReceiveMetaData) -> Self::ThreadReturn {
        ()
    }
    
    fn on_event(&mut self, _: ReceiveMetaData, _: Self::Event) -> EventHandleResult<Self> {
        EventHandleResult::TryForNextEvent
    }
    
    fn on_channel_disconnect(&mut self) -> EventHandleResult<Self> {
        EventHandleResult::StopThread(())
    }
}
