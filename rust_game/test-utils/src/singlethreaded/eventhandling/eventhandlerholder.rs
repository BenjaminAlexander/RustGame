use std::sync::{Arc, Mutex};
use log::trace;
use commons::threading::{AsyncJoin, AsyncJoinCallBackTrait, ThreadBuilder};
use commons::threading::channel::{Channel, ChannelThreadBuilder, ReceiveMetaData};
use commons::threading::eventhandling::{ChannelEvent, EventHandlerTrait, EventOrStopThread, EventHandlerSender, EventHandleResult};
use commons::threading::eventhandling::ChannelEvent::{ChannelDisconnected, ChannelEmpty, ReceivedEvent, Timeout};
use commons::time::TimeDuration;
use crate::singlethreaded::{ReceiveOrDisconnected, ReceiverLink, SingleThreadedFactory};

pub struct EventHandlerHolder<T: EventHandlerTrait, U: AsyncJoinCallBackTrait<SingleThreadedFactory, T::ThreadReturn>> {
    internal: Arc<Mutex<Option<EventHandlerHolderInternal<T, U>>>>,
    factory: SingleThreadedFactory,
}

struct EventHandlerHolderInternal<T: EventHandlerTrait, U: AsyncJoinCallBackTrait<SingleThreadedFactory, T::ThreadReturn>> {
    receiver_link: ReceiverLink<EventOrStopThread<T::Event>>,
    event_handler: T,
    join_call_back: U,
    thread_builder: ThreadBuilder<SingleThreadedFactory>,
    pending_channel_event: Option<usize>
}

impl<T: EventHandlerTrait, U: AsyncJoinCallBackTrait<SingleThreadedFactory, T::ThreadReturn>> Clone for EventHandlerHolder<T, U> {
    fn clone(&self) -> Self {
        return Self {
            internal: self.internal.clone(),
            //TODO: can we get rid of this factory?
            factory: self.factory.clone()
        }
    }
}

impl<T: EventHandlerTrait, U: AsyncJoinCallBackTrait<SingleThreadedFactory, T::ThreadReturn>> EventHandlerHolder<T, U> {

    pub fn spawn_event_handler(
        factory: SingleThreadedFactory,
        thread_builder: ChannelThreadBuilder<SingleThreadedFactory, EventOrStopThread<T::Event>>,
        event_handler: T,
        join_call_back: U) -> EventHandlerSender<SingleThreadedFactory, T::Event> {

        let (thread_builder, channel) = thread_builder.take();

        return Self::spawn_event_handler_helper(
            factory,
            thread_builder,
            channel,
            event_handler,
            join_call_back
        );
    }

    pub fn spawn_event_handler_helper(
        factory: SingleThreadedFactory,
        thread_builder: ThreadBuilder<SingleThreadedFactory>,
        channel: Channel<SingleThreadedFactory, EventOrStopThread<T::Event>>,
        event_handler: T,
        join_call_back: U) -> EventHandlerSender<SingleThreadedFactory, T::Event> {

        let (sender, receiver) = channel.take();

        let holder = Self {
            internal: Arc::new(Mutex::new(None)),
            factory: factory.clone()
        };

        let holder_clone = holder.clone();
        let receiver_link = receiver.to_consumer(move |receive_or_disconnect|{

            let holder_clone_clone = holder_clone.clone();

            factory.get_time_queue().add_event_now(move || {
                match receive_or_disconnect {
                    ReceiveOrDisconnected::Receive(receive_meta_data, event_or_stop) => {
                        holder_clone_clone.do_if_present(|internal| {
                            return internal.on_receive(&holder_clone_clone, receive_meta_data, event_or_stop);
                        });
                    }
                    ReceiveOrDisconnected::Disconnected => {
                        holder_clone_clone.do_if_present(|internal| {
                            return internal.on_channel_event(&holder_clone_clone, ChannelDisconnected);
                        });
                    }
                }
            });

            return Ok(());
        });

        let mut internal = EventHandlerHolderInternal {
            receiver_link,
            event_handler,
            join_call_back,
            thread_builder,
            pending_channel_event: None
        };

        internal.schedule_channel_empty(&holder);

        *holder.internal.lock().unwrap() = Some(internal);

        return sender;
    }

    fn do_if_present(&self, func: impl FnOnce(EventHandlerHolderInternal<T, U>) -> Option<EventHandlerHolderInternal<T, U>>) {
        let mut guard = self.internal.lock().unwrap();
        if let Some(internal) = guard.take() {
            trace!("Event Handler is still running");
            if let Some(internal) = func(internal) {
                *guard = Some(internal);
            }
        } else {
            trace!("Event Handler is not running");
        }
    }
}

impl<T: EventHandlerTrait, U: AsyncJoinCallBackTrait<SingleThreadedFactory, T::ThreadReturn>> EventHandlerHolderInternal<T, U> {

    fn cancel_pending_event(&mut self, holder: &EventHandlerHolder<T, U>) {
        match self.pending_channel_event.take() {
            None => {}
            Some(queue_event) => holder.factory.get_time_queue().remove_event(queue_event)
        }
    }

    fn schedule_channel_empty(&mut self, holder: &EventHandlerHolder<T, U>) {
        trace!("Scheduling a ChannelEmpty");
        self.cancel_pending_event(holder);

        let holder_clone = holder.clone();

        let event_id = holder.factory.get_time_queue().add_event_now(move ||{
            holder_clone.do_if_present(|mut internal|{
                internal.pending_channel_event = None;
                trace!("Executing the previously scheduled ChannelEmpty");
                return internal.on_channel_event(&holder_clone, ChannelEmpty);
            });
        });

        self.pending_channel_event = Some(event_id);
    }

    fn schedule_timeout(&mut self, holder: &EventHandlerHolder<T, U>, time_duration: TimeDuration)  {
        trace!("Scheduling a Timeout");
        self.cancel_pending_event(holder);

        let holder_clone = holder.clone();

        let event_id = holder.factory.get_time_queue().add_event_at_duration_from_now(time_duration, move || {
            holder_clone.do_if_present(|mut internal|{
                internal.pending_channel_event = None;
                trace!("Executing the previously scheduled Timeout");
                return internal.on_channel_event(&holder_clone, Timeout);
            });
        });

        self.pending_channel_event = Some(event_id);
    }

    fn on_receive(self, holder: &EventHandlerHolder<T, U>, receive_meta_data: ReceiveMetaData, event_or_stop: EventOrStopThread<T::Event>) -> Option<Self> {
        match event_or_stop {
            EventOrStopThread::Event(event) => {
                trace!("Executing a ReceivedEvent");
                return self.on_channel_event(holder, ReceivedEvent(receive_meta_data, event));
            }
            EventOrStopThread::StopThread => {
                let result = self.event_handler.on_stop(receive_meta_data);
                self.receiver_link.disconnect_receiver();
                self.join_call_back.join(AsyncJoin::new(self.thread_builder, result));
                return None;
            }
        }
    }

    fn on_channel_event(mut self, holder: &EventHandlerHolder<T, U>, event: ChannelEvent<T::Event>) -> Option<Self> {

        trace!("Event Handler: {:?}", self.thread_builder.get_name());

        match self.event_handler.on_channel_event(event) {
            EventHandleResult::WaitForNextEvent(event_handler) => {
                trace!("WaitForNextEvent");
                self.event_handler = event_handler;
                return Some(self);
            }
            EventHandleResult::WaitForNextEventOrTimeout(event_handler, timeout_duration) => {
                trace!("WaitForNextEventOrTimeout: {:?}", timeout_duration);
                self.event_handler = event_handler;
                self.schedule_timeout(&holder, timeout_duration);
                return Some(self);
            }
            EventHandleResult::TryForNextEvent(event_handler) => {
                trace!("TryForNextEvent");
                self.event_handler = event_handler;
                self.schedule_channel_empty(&holder);
                return Some(self);
            }
            EventHandleResult::StopThread(result) => {
                trace!("Join");
                self.receiver_link.disconnect_receiver();
                self.join_call_back.join(AsyncJoin::new(self.thread_builder, result));
                return None;
            }
        };
    }
}
