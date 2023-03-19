use std::mem;
use std::ops::ControlFlow::{Break, Continue};
use commons::threading::{AsyncJoin, ThreadBuilder};
use commons::threading::eventhandling::{ChannelEvent, ChannelEventResult, EventHandlerTrait, WaitOrTryForNextEvent};
use commons::time::TimeValue;
use crate::threading::eventhandling::eventhandlerholder::EventHandlerHolder::{Joined, TryForNextEvent, WaitForNextEvent, WaitForNextEventOrTimeout};

enum EventHandlerHolder<T: EventHandlerTrait, U: FnOnce(AsyncJoin<T::ThreadReturn>)> {

    WaitForNextEvent {
        event_handler: T,
        join_call_back: U,
        thread_builder: ThreadBuilder
    },
    WaitForNextEventOrTimeout {
        event_handler: T,
        join_call_back: U,
        thread_builder: ThreadBuilder,
        timeout: TimeValue
    },
    TryForNextEvent {
        event_handler: T,
        join_call_back: U,
        thread_builder: ThreadBuilder
    },
    Joined
}

impl<T: EventHandlerTrait, U: FnOnce(AsyncJoin<T::ThreadReturn>)> EventHandlerHolder<T, U> {

    fn take(&mut self) -> Self {
        return mem::replace(self, Joined);
    }

    fn on_channel_event(&mut self, event: ChannelEvent<T::Event>) {

        *self = match self.take() {
            WaitForNextEvent {
                event_handler,
                join_call_back,
                thread_builder,
            } => Self::do_channel_event(event_handler, join_call_back, thread_builder, event),
            WaitForNextEventOrTimeout {
                event_handler,
                join_call_back,
                thread_builder,
                timeout
            } => Self::do_channel_event(event_handler, join_call_back, thread_builder, event),
            TryForNextEvent {
                event_handler,
                join_call_back,
                thread_builder
            } => Self::do_channel_event(event_handler, join_call_back, thread_builder, event),

            //TODO: warn for this case
            Joined => Joined
        };
    }

    fn do_channel_event(event_handler: T, join_call_back: U, thread_builder: ThreadBuilder, event: ChannelEvent<T::Event>) -> Self {
        match event_handler.on_channel_event(event) {
            Continue(WaitOrTryForNextEvent::WaitForNextEvent(event_handler)) => {
                return WaitForNextEvent {
                    event_handler,
                    join_call_back,
                    thread_builder
                };
            }
            Continue(WaitOrTryForNextEvent::WaitForNextEventOrTimeout(event_handler, timeout_duration)) => {
                return WaitForNextEventOrTimeout {
                    event_handler,
                    join_call_back,
                    thread_builder,
                    timeout: TimeValue::now().add(timeout_duration)
                };
            }
            Continue(WaitOrTryForNextEvent::TryForNextEvent(event_handler)) => {
                return TryForNextEvent {
                    event_handler,
                    join_call_back,
                    thread_builder
                };
            }
            Break(result) => {
                join_call_back(AsyncJoin::new(thread_builder, result));
                return Joined;
            }
        };

    }
}