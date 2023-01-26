use std::ops::ControlFlow::{Break, Continue};
use log::info;
use crate::threading;
use crate::threading::channel::{TryRecvError, Receiver, ReceiveMetaData};
use crate::threading::eventhandling::{ChannelEventResult, EventHandlerTrait, EventOrStopThread};
use crate::threading::eventhandling::EventOrStopThread::{Event, StopThread};
use crate::threading::eventhandling::ChannelEvent::{ChannelDisconnected, ChannelEmpty, ReceivedEvent};
use crate::threading::eventhandling::WaitOrTryForNextEvent::{TryForNextEvent, WaitForNextEvent};

type EventReceiver<T> = Receiver<EventOrStopThread<T>>;

pub struct Thread<T: EventHandlerTrait> {
    receiver: EventReceiver<T::Event>,
    event_handler: T
}

impl<T: EventHandlerTrait> Thread<T> {

    pub(in crate::threading) fn new(receiver: EventReceiver<T::Event>, event_handler: T) -> Self {
        return Self {
            receiver,
            event_handler
        };
    }

    fn wait_for_message(message_handler: T, receiver: &EventReceiver<T::Event>) -> ChannelEventResult<T> {

        return match receiver.recv_meta_data() {
            Ok((receive_meta_data, Event(event))) => Self::on_message(message_handler, receive_meta_data, event),
            Ok((receive_meta_data, StopThread)) => Break(Self::on_stop(message_handler, receive_meta_data)),
            Err(_) => Self::on_channel_disconnected(message_handler)
        };
    }

    fn try_for_message(message_handler: T, receiver: &EventReceiver<T::Event>) -> ChannelEventResult<T> {

        return match receiver.try_recv_meta_data() {
            Ok((receive_meta_data, Event(event))) => Self::on_message(message_handler, receive_meta_data, event),
            Ok((receive_meta_data, StopThread)) => Break(Self::on_stop(message_handler, receive_meta_data)),
            Err(TryRecvError::Disconnected) => Self::on_channel_disconnected(message_handler),
            Err(TryRecvError::Empty) => Self::on_channel_empty(message_handler)
        };
    }

    fn on_message(message_handler: T, receive_meta_data: ReceiveMetaData, event: T::Event) -> ChannelEventResult<T> {
        return message_handler.on_channel_event(ReceivedEvent(receive_meta_data, event));
    }

    fn on_channel_empty(message_handler: T) -> ChannelEventResult<T> {
        return message_handler.on_channel_event(ChannelEmpty);
    }

    fn on_channel_disconnected(message_handler: T) -> ChannelEventResult<T> {
        info!("The receiver channel has been disconnected.");
        return message_handler.on_channel_event(ChannelDisconnected);
    }

    fn on_stop(message_handler: T, receive_meta_data: ReceiveMetaData) -> T::ThreadReturn {
        info!("The MessageHandlingThread has received a message commanding it to stop.");
        return message_handler.on_stop(receive_meta_data);
    }
}

impl<T: EventHandlerTrait> threading::Thread for Thread<T> {
    type ReturnType = T::ThreadReturn;

    fn run(self) -> Self::ReturnType {

        let mut wait_or_try = TryForNextEvent(self.event_handler);

        loop {

            let result = match wait_or_try {
                WaitForNextEvent(message_handler) => Self::wait_for_message(message_handler, &self.receiver),
                TryForNextEvent(message_handler) => Self::try_for_message(message_handler, &self.receiver),
            };

            wait_or_try = match result {
                Continue(wait_or_try) => wait_or_try,
                Break(return_value) => {
                    return return_value;
                }
            };
        }
    }
}
