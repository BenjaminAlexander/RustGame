use std::ops::ControlFlow::{Break, Continue};
use log::info;
use crate::threading::eventhandling::{ChannelEventResult, EventHandlerTrait, EventOrStopThread, Sender, ReceivedEventHolder, SentEventHolder};
use crate::threading::eventhandling::EventOrStopThread::{Event, StopThread};
use crate::threading::{message_channel, build_thread as build_base_thread, Thread as BaseThread, ValueReceiver, ValueTryRecvError};
use crate::threading::eventhandling::ChannelEvent::{ChannelDisconnected, ChannelEmpty, ReceivedEvent};
use crate::threading::eventhandling::threadbuilder::ThreadBuilder;
use crate::threading::eventhandling::WaitOrTryForNextEvent::{TryForNextEvent, WaitForNextEvent};

type EventReceiver<T> = ValueReceiver<EventOrStopThread<T>>;

pub(super) struct Thread<T: EventHandlerTrait> {
    pub(super) receiver: EventReceiver<T>,
    pub(super) event_handler: T
}

impl<T: EventHandlerTrait> Thread<T> {

    fn wait_for_message(message_handler: T, receiver: &EventReceiver<T>) -> ChannelEventResult<T> {

        return match receiver.recv() {
            Ok(Event(sent_event_holder)) => Self::on_message(message_handler, sent_event_holder),
            Ok(StopThread) => Break(Self::on_stop(message_handler)),
            Err(_) => Self::on_channel_disconnected(message_handler)
        };
    }

    fn try_for_message(message_handler: T, receiver: &EventReceiver<T>) -> ChannelEventResult<T> {

        return match receiver.try_recv() {
            Ok(Event(sent_event_holder)) => Self::on_message(message_handler, sent_event_holder),
            Ok(StopThread) => Break(Self::on_stop(message_handler)),
            Err(ValueTryRecvError::Disconnected) => Self::on_channel_disconnected(message_handler),
            Err(ValueTryRecvError::Empty) => Self::on_channel_empty(message_handler)
        };
    }

    fn on_message(message_handler: T, sent_event_holder: SentEventHolder<T>) -> ChannelEventResult<T> {
        return message_handler.on_channel_event(ReceivedEvent(ReceivedEventHolder { sent_event_holder }));
    }

    fn on_channel_empty(message_handler: T) -> ChannelEventResult<T> {
        return message_handler.on_channel_event(ChannelEmpty);
    }

    fn on_channel_disconnected(message_handler: T) -> ChannelEventResult<T> {
        info!("The receiver channel has been disconnected.");
        return message_handler.on_channel_event(ChannelDisconnected);
    }

    fn on_stop(message_handler: T) -> T::ThreadReturn {
        info!("The MessageHandlingThread has received a message commanding it to stop.");
        return message_handler.on_stop();
    }
}

impl<T: EventHandlerTrait> BaseThread for Thread<T> {
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