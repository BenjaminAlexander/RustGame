use crate::real_time::simulation::receiver_link::{
    ReceiveOrDisconnected,
    ReceiverLink,
};
use crate::real_time::simulation::single_threaded_receiver::SingleThreadedReceiver;
use crate::real_time::simulation::SingleThreadedFactory;
use crate::real_time::{
    EventHandleResult,
    EventOrStopThread,
    HandleEvent,
    ReceiveMetaData,
};
use crate::time::TimeDuration;
use log::trace;
use std::sync::{
    Arc,
    Mutex,
};

pub fn spawn_event_handler<T: HandleEvent, U: FnOnce(T::ThreadReturn) + Send + 'static>(
    thread_name: String,
    receiver: SingleThreadedReceiver<EventOrStopThread<T::Event>>,
    event_handler: T,
    join_call_back: U,
) {
    let holder = EventHandlerHolder::<T, U> {
        internal: Arc::new(Mutex::new(None)),
        factory: receiver.get_factory().clone(),
    };

    let holder_clone = holder.clone();
    let factory = receiver.get_factory().clone();
    let receiver_link = receiver.to_consumer(move |receive_or_disconnect| {
        let holder_clone_clone = holder_clone.clone();

        factory
            .get_time_queue()
            .add_event_now(move || match receive_or_disconnect {
                ReceiveOrDisconnected::Receive(receive_meta_data, event_or_stop) => {
                    holder_clone_clone.do_if_present(|internal| {
                        return internal.on_receive(
                            &holder_clone_clone,
                            receive_meta_data,
                            event_or_stop,
                        );
                    });
                }
                ReceiveOrDisconnected::Disconnected => {
                    holder_clone_clone.do_if_present(|mut internal| {
                        let event_handle_result = internal.event_handler.on_channel_disconnect();
                        return internal
                            .handle_event_handle_result(&holder_clone_clone, event_handle_result);
                    });
                }
            });

        return Ok(());
    });

    let mut internal = EventHandlerHolderInternal {
        receiver_link,
        event_handler,
        join_call_back,
        thread_name,
        pending_channel_event: None,
    };

    internal.schedule_channel_empty(&holder);

    *holder.internal.lock().unwrap() = Some(internal);
}

struct EventHandlerHolder<T: HandleEvent, U: FnOnce(T::ThreadReturn) + Send + 'static> {
    internal: Arc<Mutex<Option<EventHandlerHolderInternal<T, U>>>>,
    factory: SingleThreadedFactory,
}

struct EventHandlerHolderInternal<T: HandleEvent, U: FnOnce(T::ThreadReturn) + Send + 'static> {
    receiver_link: ReceiverLink<EventOrStopThread<T::Event>>,
    event_handler: T,
    join_call_back: U,
    thread_name: String,
    pending_channel_event: Option<usize>,
}

impl<T: HandleEvent, U: FnOnce(T::ThreadReturn) + Send + 'static> Clone
    for EventHandlerHolder<T, U>
{
    fn clone(&self) -> Self {
        return Self {
            internal: self.internal.clone(),
            //TODO: can we get rid of this factory?
            factory: self.factory.clone(),
        };
    }
}

impl<T: HandleEvent, U: FnOnce(T::ThreadReturn) + Send + 'static> EventHandlerHolder<T, U> {
    fn do_if_present(
        &self,
        func: impl FnOnce(EventHandlerHolderInternal<T, U>) -> Option<EventHandlerHolderInternal<T, U>>,
    ) {
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

impl<T: HandleEvent, U: FnOnce(T::ThreadReturn) + Send + 'static> EventHandlerHolderInternal<T, U> {
    fn cancel_pending_event(&mut self, holder: &EventHandlerHolder<T, U>) {
        match self.pending_channel_event.take() {
            None => {}
            Some(queue_event) => holder.factory.get_time_queue().remove_event(queue_event),
        }
    }

    fn schedule_channel_empty(&mut self, holder: &EventHandlerHolder<T, U>) {
        trace!("Scheduling a ChannelEmpty");
        self.cancel_pending_event(holder);

        let holder_clone = holder.clone();

        let event_id = holder.factory.get_time_queue().add_event_now(move || {
            holder_clone.do_if_present(|mut internal| {
                internal.pending_channel_event = None;
                trace!("Executing the previously scheduled ChannelEmpty");
                let event_handle_result = internal.event_handler.on_channel_empty();
                return internal.handle_event_handle_result(&holder_clone, event_handle_result);
            });
        });

        self.pending_channel_event = Some(event_id);
    }

    fn schedule_timeout(&mut self, holder: &EventHandlerHolder<T, U>, time_duration: TimeDuration) {
        trace!("Scheduling a Timeout");
        self.cancel_pending_event(holder);

        let holder_clone = holder.clone();

        let event_id = holder
            .factory
            .get_time_queue()
            .add_event_at_duration_from_now(time_duration, move || {
                holder_clone.do_if_present(|mut internal| {
                    internal.pending_channel_event = None;
                    trace!("Executing the previously scheduled Timeout");
                    let event_handle_result = internal.event_handler.on_timeout();
                    return internal.handle_event_handle_result(&holder_clone, event_handle_result);
                });
            });

        self.pending_channel_event = Some(event_id);
    }

    fn on_receive(
        mut self,
        holder: &EventHandlerHolder<T, U>,
        receive_meta_data: ReceiveMetaData,
        event_or_stop: EventOrStopThread<T::Event>,
    ) -> Option<Self> {
        match event_or_stop {
            EventOrStopThread::Event(event) => {
                trace!("Executing a ReceivedEvent");
                let event_handle_result = self.event_handler.on_event(receive_meta_data, event);
                return self.handle_event_handle_result(&holder, event_handle_result);
            }
            EventOrStopThread::StopThread => {
                let result = self.event_handler.on_stop(receive_meta_data);
                self.receiver_link.disconnect_receiver();
                (self.join_call_back)(result);
                return None;
            }
        }
    }

    fn handle_event_handle_result(
        mut self,
        holder: &EventHandlerHolder<T, U>,
        event_handle_result: EventHandleResult<T>,
    ) -> Option<Self> {
        trace!("Event Handler: {:?}", self.thread_name);

        match event_handle_result {
            EventHandleResult::WaitForNextEvent => {
                trace!("WaitForNextEvent");
                return Some(self);
            }
            EventHandleResult::WaitForNextEventOrTimeout(timeout_duration) => {
                trace!("WaitForNextEventOrTimeout: {:?}", timeout_duration);
                self.schedule_timeout(&holder, timeout_duration);
                return Some(self);
            }
            EventHandleResult::TryForNextEvent => {
                trace!("TryForNextEvent");
                self.schedule_channel_empty(&holder);
                return Some(self);
            }
            EventHandleResult::StopThread(result) => {
                trace!("Join");
                self.receiver_link.disconnect_receiver();
                (self.join_call_back)(result);
                return None;
            }
        };
    }
}
