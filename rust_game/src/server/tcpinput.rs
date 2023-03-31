use log::error;
use crate::messaging::{ToServerMessageTCP};
use std::ops::ControlFlow::{Break, Continue};
use commons::net::TcpReceiverTrait;
use commons::threading::channel::ReceiveMetaData;
use commons::threading::listener::{ChannelEvent, ListenedOrDidNotListen, ListenerEventResult, ListenerTrait, ListenResult};
use crate::interface::{GameFactoryTrait, TcpReceiver};

pub struct TcpInput<GameFactory: GameFactoryTrait> {
    tcp_receiver: TcpReceiver<GameFactory>
}

impl<GameFactory: GameFactoryTrait> TcpInput<GameFactory> {

    pub fn new(tcp_receiver: TcpReceiver<GameFactory>) -> Self {
        return Self {
            tcp_receiver
        };
    }
}

impl<GameFactory: GameFactoryTrait> ListenerTrait for TcpInput<GameFactory> {
    type Event = ();
    type ThreadReturn = ();
    type ListenFor = ToServerMessageTCP;

    fn listen(mut self) -> ListenResult<Self> {
        let result = self.tcp_receiver.read();

        match result {
            Ok(message) => {
                return Continue(ListenedOrDidNotListen::Listened(self, message));
            }
            Err(error) => {
                error!("rmp_serde Error: {:?}", error);
                return Continue(ListenedOrDidNotListen::DidNotListen(self));
            }
        }
    }

    fn on_channel_event(self, event: ChannelEvent<Self>) -> ListenerEventResult<Self> {
        match event {
            ChannelEvent::ChannelEmptyAfterListen(_, value) => {
                match value {

                };

                return Continue(self);
            }
            ChannelEvent::ReceivedEvent(_, ()) => Continue(self),
            ChannelEvent::ChannelDisconnected => Break(())
        }
    }

    fn on_stop(self, _: ReceiveMetaData) -> Self::ThreadReturn { () }
}