use std::net::TcpStream;

use log::{error, info};
use rmp_serde::decode::Error;

use crate::messaging::{ToServerMessageTCP};
use crate::threading::{ChannelThread, OldReceiver, ThreadAction};
use std::io;
use std::ops::ControlFlow::{Break, Continue};
use std::sync::mpsc::TryRecvError;
use crate::threading::eventhandling::ReceivedEventHolder;
use crate::threading::listener::{ChannelEvent, ListenedOrDidNotListen, ListenerEventResult, ListenerTrait, ListenResult};

pub struct TcpInput {
    tcp_stream: TcpStream
}

impl TcpInput {

    pub fn new(tcp_stream: &TcpStream) -> io::Result<Self> {
        Ok(Self {tcp_stream: tcp_stream.try_clone()?})
    }
}

impl ListenerTrait for TcpInput {
    type Event = ();
    type ThreadReturn = ();
    type ListenFor = ToServerMessageTCP;

    fn listen(self) -> ListenResult<Self> {
        let result: Result<ToServerMessageTCP, Error> = rmp_serde::from_read(&self.tcp_stream);

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
            ChannelEvent::ChannelEmptyAfterListen(listened_value_holder) => {
                match listened_value_holder.move_value() {

                };

                return Continue(self);
            }
            ChannelEvent::ReceivedEvent(received_event_holder) => {
                match received_event_holder.move_event() {
                    () => Continue(self)
                }
            }
            ChannelEvent::ChannelDisconnected => Break(self.on_stop())
        }
    }

    fn on_stop(self) -> Self::ThreadReturn { () }
}