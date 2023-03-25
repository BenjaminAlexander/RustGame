use std::ops::ControlFlow::{Break, Continue};
use std::sync::{Arc, Mutex};
use commons::threading::{AsyncJoin, ThreadBuilder};
use commons::threading::channel::{ReceiveMetaData, SendMetaData};
use commons::threading::eventhandling::{ChannelEvent, EventHandlerTrait, WaitOrTryForNextEvent};
use commons::time::{TimeDuration, TimeValue};
use crate::singlethreaded::SingleThreadedFactory;

pub struct EventHandlerHolder<T: EventHandlerTrait, U: FnOnce(AsyncJoin<SingleThreadedFactory, T::ThreadReturn>) + Send + 'static> {
    internal: Arc<Mutex<Option<EventHandlerHolderInternal<T, U>>>>,
    factory: SingleThreadedFactory,
}

struct EventHandlerHolderInternal<T: EventHandlerTrait, U: FnOnce(AsyncJoin<SingleThreadedFactory, T::ThreadReturn>) + Send + 'static> {
    event_handler: T,
    join_call_back: U,
    thread_builder: ThreadBuilder<SingleThreadedFactory>,
    pending_channel_event: Option<usize>
}

impl<T: EventHandlerTrait, U: FnOnce(AsyncJoin<SingleThreadedFactory, T::ThreadReturn>) + Send + 'static> Clone for EventHandlerHolder<T, U> {
    fn clone(&self) -> Self {
        return Self {
            internal: self.internal.clone(),
            factory: self.factory.clone()
        }
    }
}

impl<T: EventHandlerTrait, U: FnOnce(AsyncJoin<SingleThreadedFactory, T::ThreadReturn>) + Send + 'static> EventHandlerHolder<T, U> {

    pub fn new(factory: SingleThreadedFactory, thread_builder: ThreadBuilder<SingleThreadedFactory>, event_handler: T, join_call_back: U) -> Self {

        let internal = EventHandlerHolderInternal {
            event_handler,
            join_call_back,
            thread_builder,
            pending_channel_event: None
        };

        let result = Self {
            internal: Arc::new(Mutex::new(Some(internal))),
            factory
        };

        result.send_channel_event_now(ChannelEvent::ChannelEmpty);

        return result;
    }

    pub fn send_channel_event_now(&self, event: ChannelEvent<T::Event>) -> usize {
        let now = self.factory.get_simulated_time_source().now();
        return self.send_channel_event_at_time(now, event);
    }

    pub fn send_channel_event_at_duration_from_now(&self, time_duration: TimeDuration, event: ChannelEvent<T::Event>) -> usize {
        let time_value = self.factory.get_simulated_time_source().now().add(time_duration);
        return self.send_channel_event_at_time(time_value, event);
    }

    pub fn send_channel_event_at_time(&self, time_value: TimeValue, event: ChannelEvent<T::Event>) -> usize {
        let self_clone = self.clone();
        return self.factory.get_time_queue().add_event_at_time(time_value, move || {

            let taken = self_clone.internal.lock().unwrap().take();

            if let Some(mut internal) = taken {

                match internal.pending_channel_event.take() {
                    None => {}
                    Some(queue_event) => self_clone.factory.get_time_queue().remove_event(queue_event)
                }

                match internal.event_handler.on_channel_event(event) {
                    Continue(WaitOrTryForNextEvent::WaitForNextEvent(event_handler)) => {
                        internal.event_handler = event_handler;
                        self_clone.internal.lock().unwrap().replace(internal);
                    }
                    Continue(WaitOrTryForNextEvent::WaitForNextEventOrTimeout(event_handler, timeout_duration)) => {
                        internal.event_handler = event_handler;
                        let queue_event = self_clone.send_channel_event_at_duration_from_now(timeout_duration, ChannelEvent::Timeout);
                        internal.pending_channel_event = Some(queue_event);
                        self_clone.internal.lock().unwrap().replace(internal);
                    }
                    Continue(WaitOrTryForNextEvent::TryForNextEvent(event_handler)) => {
                        internal.event_handler = event_handler;
                        let queue_event = self_clone.send_channel_event_now(ChannelEvent::ChannelEmpty);
                        internal.pending_channel_event = Some(queue_event);
                        self_clone.internal.lock().unwrap().replace(internal);
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
        let send_meta_data = SendMetaData::new(&self.factory);
        let receive_meta_data = ReceiveMetaData::new(&self.factory, send_meta_data);
        self.send_channel_event_now(ChannelEvent::ReceivedEvent(receive_meta_data, event));
    }

    pub fn send_stop(&self) {
        let self_clone = self.clone();
        let send_meta_data = SendMetaData::new(&self.factory);
        let receive_meta_data = ReceiveMetaData::new(&self.factory, send_meta_data);
        self.factory.get_time_queue().add_event_now(move || {
            if let Some(internal) = self_clone.internal.lock().unwrap().take() {
                let result = internal.event_handler.on_stop(receive_meta_data);
                (internal.join_call_back)(AsyncJoin::new(internal.thread_builder, result));
            }
        });
    }
}