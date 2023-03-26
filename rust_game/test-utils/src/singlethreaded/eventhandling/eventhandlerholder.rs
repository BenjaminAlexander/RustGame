use std::ops::ControlFlow::{Break, Continue};
use std::sync::{Arc, Mutex};
use commons::threading::{AsyncJoin, AsyncJoinCallBackTrait, ThreadBuilder};
use commons::threading::channel::{Receiver, TryRecvError};
use commons::threading::eventhandling::{ChannelEvent, EventHandlerTrait, EventOrStopThread, WaitOrTryForNextEvent};
use commons::threading::eventhandling::ChannelEvent::{ChannelDisconnected, ChannelEmpty, ReceivedEvent, Timeout};
use commons::time::TimeDuration;
use crate::singlethreaded::SingleThreadedFactory;

pub struct EventHandlerHolder<T: EventHandlerTrait, U: AsyncJoinCallBackTrait<SingleThreadedFactory, T::ThreadReturn>> {
    internal: Arc<Mutex<Option<EventHandlerHolderInternal<T, U>>>>,
    factory: SingleThreadedFactory,
}

struct EventHandlerHolderInternal<T: EventHandlerTrait, U: AsyncJoinCallBackTrait<SingleThreadedFactory, T::ThreadReturn>> {
    receiver: Receiver<EventOrStopThread<T::Event>>,
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

    pub fn new(factory: SingleThreadedFactory, thread_builder: ThreadBuilder<SingleThreadedFactory>, receiver: Receiver<EventOrStopThread<T::Event>>, event_handler: T, join_call_back: U) -> Self {

        let internal = EventHandlerHolderInternal {
            receiver,
            event_handler,
            join_call_back,
            thread_builder,
            pending_channel_event: None
        };

        let result = Self {
            internal: Arc::new(Mutex::new(Some(internal))),
            factory
        };

        result.schedule_channel_empty();

        return result;
    }

    //TODO: maybe keep track of events in the TimeQueue and remove them when the handler joins
    pub fn on_send(&self) {
        let self_clone = self.clone();
        self.factory.get_time_queue().add_event_now(move || {
            let mut guard = self_clone.internal.lock().unwrap();
            if let Some(mut internal) = guard.take() {

                internal.cancel_pending_event(&self_clone);

                if let Some(internal) = internal.try_receive(&self_clone) {
                    *guard = Some(internal);
                }
            }
        });
    }

    fn on_channel_event(&self, event: ChannelEvent<T::Event>) {
        let mut guard = self.internal.lock().unwrap();
        if let Some(internal) = guard.take() {
            if let Some(internal) = internal.on_channel_event(&self, event) {
                *guard = Some(internal);
            }
        }
    }

    fn schedule_channel_empty(&self) -> usize {
        let self_clone = self.clone();
        return self.factory.get_time_queue().add_event_now(move ||{
            self_clone.on_channel_event(ChannelEmpty);
        });
    }

    fn schedule_timeout(&self, time_duration: TimeDuration) -> usize {
        let self_clone = self.clone();
        return self.factory.get_time_queue().add_event_at_duration_from_now(time_duration, move || {
            self_clone.on_channel_event(Timeout);
        });
    }
}

impl<T: EventHandlerTrait, U: AsyncJoinCallBackTrait<SingleThreadedFactory, T::ThreadReturn>> EventHandlerHolderInternal<T, U> {

    fn cancel_pending_event(&mut self, holder: &EventHandlerHolder<T, U>) {
        match self.pending_channel_event.take() {
            None => {}
            Some(queue_event) => holder.factory.get_time_queue().remove_event(queue_event)
        }
    }

    fn try_receive(mut self, holder: &EventHandlerHolder<T, U>) -> Option<Self> {
        match self.receiver.try_recv_meta_data(&holder.factory) {
            Ok((receive_meta_data, EventOrStopThread::Event(event))) => {
                return self.on_channel_event(holder, ReceivedEvent(receive_meta_data, event));
            }
            Ok((receive_meta_data, EventOrStopThread::StopThread)) => {
                let result = self.event_handler.on_stop(receive_meta_data);
                self.join_call_back.join(AsyncJoin::new(self.thread_builder, result));
                return None;
            }
            Err(TryRecvError::Empty) => panic!("on_send was called when there is nothing in the channel"),
            Err(TryRecvError::Disconnected) => {
                return self.on_channel_event(holder, ChannelDisconnected);
            }
        };
    }

    fn on_channel_event(mut self, holder: &EventHandlerHolder<T, U>, event: ChannelEvent<T::Event>) -> Option<Self> {
        match self.event_handler.on_channel_event(event) {
            Continue(WaitOrTryForNextEvent::WaitForNextEvent(event_handler)) => {
                self.event_handler = event_handler;
                return Some(self);
            }
            Continue(WaitOrTryForNextEvent::WaitForNextEventOrTimeout(event_handler, timeout_duration)) => {
                self.event_handler = event_handler;
                let queue_event = holder.schedule_timeout(timeout_duration);
                self.pending_channel_event = Some(queue_event);
                return Some(self);
            }
            Continue(WaitOrTryForNextEvent::TryForNextEvent(event_handler)) => {
                self.event_handler = event_handler;
                let queue_event = holder.schedule_channel_empty();
                self.pending_channel_event = Some(queue_event);
                return Some(self);
            }
            Break(result) => {
                self.join_call_back.join(AsyncJoin::new(self.thread_builder, result));
                return None;
            }
        };
    }
}