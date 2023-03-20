use std::cell::RefCell;
use std::ops::ControlFlow::{Break, Continue};
use std::rc::Rc;
use commons::threading::{AsyncJoin, ThreadBuilder};
use commons::threading::channel::{ReceiveMetaData, SendMetaData};
use commons::threading::eventhandling::{ChannelEvent, EventHandlerTrait, WaitOrTryForNextEvent};
use commons::time::{TimeDuration, TimeSource, TimeValue};
use crate::singlethreaded::TimeQueue;

pub struct EventHandlerHolder<T: EventHandlerTrait, U: FnOnce(AsyncJoin<T::ThreadReturn>) + 'static> {
    internal: Rc<RefCell<Option<EventHandlerHolderInternal<T, U>>>>,
    queue: TimeQueue,
}

struct EventHandlerHolderInternal<T: EventHandlerTrait, U: FnOnce(AsyncJoin<T::ThreadReturn>) + 'static> {
    event_handler: T,
    join_call_back: U,
    thread_builder: ThreadBuilder,
    pending_channel_event: Option<usize>
}

impl<T: EventHandlerTrait, U: FnOnce(AsyncJoin<T::ThreadReturn>) + 'static> Clone for EventHandlerHolder<T, U> {
    fn clone(&self) -> Self {
        return Self {
            internal: self.internal.clone(),
            queue: self.queue.clone()
        }
    }
}

impl<T: EventHandlerTrait, U: FnOnce(AsyncJoin<T::ThreadReturn>) + 'static> EventHandlerHolder<T, U> {

    pub fn new(queue: TimeQueue, thread_builder: ThreadBuilder, event_handler: T, join_call_back: U) -> Self {

        let internal = EventHandlerHolderInternal {
            event_handler,
            join_call_back,
            thread_builder,
            pending_channel_event: None
        };

        let result = Self {
            internal: Rc::new(RefCell::new(Some(internal))),
            queue: queue.clone()
        };

        result.send_channel_event_now(ChannelEvent::ChannelEmpty);

        return result;
    }

    pub fn send_channel_event_now(&self, event: ChannelEvent<T::Event>) -> usize {
        let now = self.queue.get_time_source().now();
        return self.send_channel_event_at_time(now, event);
    }

    pub fn send_channel_event_at_duration_from_now(&self, time_duration: TimeDuration, event: ChannelEvent<T::Event>) -> usize {
        let time_value = self.queue.get_time_source().now().add(time_duration);
        return self.send_channel_event_at_time(time_value, event);
    }

    pub fn send_channel_event_at_time(&self, time_value: TimeValue, event: ChannelEvent<T::Event>) -> usize {
        let self_clone = self.clone();
        return self.queue.add_event_at_time(time_value, move || {

            let taken = self_clone.internal.take();

            if let Some(mut internal) = taken {

                match internal.pending_channel_event.take() {
                    None => {}
                    Some(queue_event) => self_clone.queue.remove_event(queue_event)
                }

                match internal.event_handler.on_channel_event(event) {
                    Continue(WaitOrTryForNextEvent::WaitForNextEvent(event_handler)) => {
                        internal.event_handler = event_handler;
                        self_clone.internal.replace(Some(internal));
                    }
                    Continue(WaitOrTryForNextEvent::WaitForNextEventOrTimeout(event_handler, timeout_duration)) => {
                        internal.event_handler = event_handler;
                        let queue_event = self_clone.send_channel_event_at_duration_from_now(timeout_duration, ChannelEvent::Timeout);
                        internal.pending_channel_event = Some(queue_event);
                        self_clone.internal.replace(Some(internal));
                    }
                    Continue(WaitOrTryForNextEvent::TryForNextEvent(event_handler)) => {
                        internal.event_handler = event_handler;
                        let queue_event = self_clone.send_channel_event_now(ChannelEvent::ChannelEmpty);
                        internal.pending_channel_event = Some(queue_event);
                        self_clone.internal.replace(Some(internal));
                    }
                    Break(result) => {
                        (internal.join_call_back)(AsyncJoin::new(internal.thread_builder, result));
                    }
                }
            } else {
                panic!();
            }
        });
    }

    pub fn send_event(&self, event: T::Event) {
        let send_meta_data = SendMetaData::new();
        let receive_meta_data = ReceiveMetaData::new(send_meta_data);
        self.send_channel_event_now(ChannelEvent::ReceivedEvent(receive_meta_data, event));
    }

    pub fn send_stop(&self) {
        let self_clone = self.clone();
        let send_meta_data = SendMetaData::new();
        let receive_meta_data = ReceiveMetaData::new(send_meta_data);
        self.queue.add_event_now(move || {
            if let Some(internal) = self_clone.internal.take() {
                let result = internal.event_handler.on_stop(receive_meta_data);
                (internal.join_call_back)(AsyncJoin::new(internal.thread_builder, result));
            }
        });
    }
}