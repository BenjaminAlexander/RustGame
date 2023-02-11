use std::ops::ControlFlow::{Break, Continue};
use log::info;
use crate::gametime::TimeValue;
use crate::{threading, TimeDuration};
use crate::threading::channel::{TryRecvError, Receiver, ReceiveMetaData};
use crate::threading::eventhandling::{ChannelEventResult, EventHandlerTrait, EventOrStopThread};
use crate::threading::eventhandling::EventOrStopThread::{Event, StopThread};
use crate::threading::eventhandling::ChannelEvent::{ChannelDisconnected, ChannelEmpty, ReceivedEvent};
use crate::threading::eventhandling::WaitOrTryForNextEvent::{TryForNextEvent, WaitForNextEvent};

type EventReceiver<T> = Receiver<EventOrStopThread<T>>;

struct Stats {
    busy_time: TimeValue,
    longest_busy_duration: Option<TimeDuration>
}

impl Stats {
    fn new() -> Self {
        return Self {
            busy_time: TimeValue::now(),
            longest_busy_duration: None
        }
    }

    fn before_wait(&mut self) {
        let busy_duration = TimeValue::now().duration_since(&self.busy_time);

        if let Some(current_longest) = self.longest_busy_duration.as_ref() {
            if busy_duration > *current_longest {
                info!("New longest busy duration: {:?}", busy_duration);
                self.longest_busy_duration = Some(busy_duration);
            }
        } else {
            self.longest_busy_duration = Some(busy_duration);
        }
    }

    fn after_wait(&mut self) {
        self.busy_time = TimeValue::now();
    }
}

pub struct EventHandlerThread<T: EventHandlerTrait> {
    receiver: EventReceiver<T::Event>,
    event_handler: T
}

impl<T: EventHandlerTrait> EventHandlerThread<T> {

    pub(in crate::threading) fn new(receiver: EventReceiver<T::Event>, event_handler: T) -> Self {
        return Self {
            receiver,
            event_handler
        };
    }

    fn wait_for_message(message_handler: T, receiver: &mut EventReceiver<T::Event>, stats: &mut Stats) -> ChannelEventResult<T> {

        stats.before_wait();

        let recv_result = receiver.recv_meta_data();

        stats.after_wait();

        return match recv_result {
            Ok((receive_meta_data, Event(event))) => Self::on_message(message_handler, receive_meta_data, event),
            Ok((receive_meta_data, StopThread)) => Break(Self::on_stop(message_handler, receive_meta_data)),
            Err(_) => Self::on_channel_disconnected(message_handler)
        };
    }

    fn try_for_message(message_handler: T, receiver: &mut EventReceiver<T::Event>) -> ChannelEventResult<T> {

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

impl<T: EventHandlerTrait> threading::Thread for EventHandlerThread<T> {
    type ReturnType = T::ThreadReturn;

    fn run(mut self) -> Self::ReturnType {

        let mut stats = Stats::new();

        let mut wait_or_try = TryForNextEvent(self.event_handler);

        loop {

            let result = match wait_or_try {
                WaitForNextEvent(message_handler) => Self::wait_for_message(message_handler, &mut self.receiver, &mut stats),
                TryForNextEvent(message_handler) => Self::try_for_message(message_handler, &mut self.receiver),
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

