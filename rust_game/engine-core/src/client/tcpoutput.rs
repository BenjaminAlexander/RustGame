use commons::net::TcpStream;
use commons::threading::channel::ReceiveMetaData;
use commons::threading::eventhandling::ChannelEvent::{
    ChannelDisconnected,
    ChannelEmpty,
    ReceivedEvent,
    Timeout,
};
use commons::threading::eventhandling::{
    ChannelEvent,
    EventHandleResult,
    EventHandlerTrait,
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

impl EventHandlerTrait for TcpOutput {
    type Event = ();
    type ThreadReturn = ();

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> EventHandleResult<Self> {
        match channel_event {
            ReceivedEvent(_, ()) => EventHandleResult::TryForNextEvent(self),
            Timeout => EventHandleResult::WaitForNextEvent(self),
            ChannelEmpty => EventHandleResult::WaitForNextEvent(self),
            ChannelDisconnected => EventHandleResult::StopThread(()),
        }
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> Self::ThreadReturn {
        ()
    }
}
