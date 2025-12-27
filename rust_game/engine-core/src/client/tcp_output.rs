use std::io::Error;

use commons::real_time::{
    EventHandleResult,
    EventHandlerBuilder,
    EventSender,
    Factory,
    HandleEvent,
    ReceiveMetaData,
    net::tcp::TcpStream,
};

pub struct TcpOutput {
    sender: EventSender<()>,
}

impl TcpOutput {
    pub fn new(factory: &Factory, tcp_stream: TcpStream) -> Result<Self, Error> {
        let event_handler = EventHandler {
            _tcp_stream: tcp_stream,
        };

        let sender =
            EventHandlerBuilder::new_thread(factory, "ClientTcpOutput".to_string(), event_handler)?;

        Ok(Self { sender })
    }

    pub fn stop(&self) -> Result<(), ()> {
        self.sender.send_stop_thread()
    }
}

//TODO: Send response to time messages to calculate ping
struct EventHandler {
    _tcp_stream: TcpStream,
}

impl HandleEvent for EventHandler {
    type Event = ();
    type ThreadReturn = ();

    fn on_event(&mut self, _: ReceiveMetaData, _: Self::Event) -> EventHandleResult {
        EventHandleResult::TryForNextEvent
    }

    fn on_stop_self(self) -> Self::ThreadReturn {
        ()
    }
}
