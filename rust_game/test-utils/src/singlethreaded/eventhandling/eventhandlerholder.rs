use std::collections::VecDeque;
use std::ops::ControlFlow::{Break, Continue};
use std::sync::{Arc, Mutex};
use log::trace;
use commons::threading::{AsyncJoin, AsyncJoinCallBackTrait, ThreadBuilder};
use commons::threading::channel::{Channel, ChannelThreadBuilder, ReceiveMetaData, Receiver, TryRecvError};
use commons::threading::eventhandling::{ChannelEvent, EventHandlerTrait, EventOrStopThread, Sender, WaitOrTryForNextEvent};
use commons::threading::eventhandling::ChannelEvent::{ChannelDisconnected, ChannelEmpty, ReceivedEvent, Timeout};
use commons::time::TimeDuration;
use crate::singlethreaded::SingleThreadedFactory;

pub struct EventHandlerHolder<T: EventHandlerTrait, U: AsyncJoinCallBackTrait<SingleThreadedFactory, T::ThreadReturn>> {
    internal: Arc<Mutex<Option<EventHandlerHolderInternal<T, U>>>>,
    factory: SingleThreadedFactory,
}

struct EventHandlerHolderInternal<T: EventHandlerTrait, U: AsyncJoinCallBackTrait<SingleThreadedFactory, T::ThreadReturn>> {
    receiver: Receiver<SingleThreadedFactory, EventOrStopThread<T::Event>>,
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
        join_call_back: U) -> Sender<SingleThreadedFactory, T::Event> {

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
        join_call_back: U) -> Sender<SingleThreadedFactory, T::Event> {

        let (sender, mut receiver) = channel.take();

        //Empty the channel
        let mut events = VecDeque::<(ReceiveMetaData, EventOrStopThread<T::Event>)>::new();

        let try_recv_error;
        loop {
            match receiver.try_recv_meta_data() {
                Ok(received) => events.push_back(received),
                Err(error) => {
                    try_recv_error = error;
                    break
                }
            }
        }


        let internal = EventHandlerHolderInternal {
            receiver,
            event_handler,
            join_call_back,
            thread_builder,
            pending_channel_event: None
        };

        let holder = Self {
            internal: Arc::new(Mutex::new(Some(internal))),
            factory: factory.clone()
        };

        loop {
            match events.pop_front() {
                None => break,
                Some((receive_meta_data, event_or_stop)) => {

                    let holder_clone = holder.clone();

                    //TODO: maybe keep track of these events and remove them if it joins
                    factory.get_time_queue().add_event_now(move ||{
                        holder_clone.do_if_present(|internal|{
                            return internal.on_receive(&holder_clone, receive_meta_data, event_or_stop);
                        });
                    });

                }
            }
        }

        match try_recv_error {
            TryRecvError::Empty => {
                holder.schedule_channel_empty();
            }
            TryRecvError::Disconnected => {
                let holder_clone = holder.clone();
                //TODO: maybe keep track of these events and remove them if it joins
                factory.get_time_queue().add_event_now(move||{
                    holder_clone.do_if_present(|internal|{
                        trace!("Executing ChannelDisconnected identified during spawn.");
                        return internal.on_channel_event(&holder_clone, ChannelDisconnected);
                    });
                });
            }
        }


        sender.set_on_send(move ||{

            let holder_clone = holder.clone();
            //TODO: maybe keep track of events in the TimeQueue and remove them when the handler joins
            factory.get_time_queue().add_event_now(move || {
                holder_clone.do_if_present(|mut internal| {
                    internal.cancel_pending_event(&holder_clone);
                    return internal.try_receive(&holder_clone);
                });
            });
        });

        return sender;
    }

    //TODO: don't return value
    fn schedule_channel_empty(&self) {
        trace!("Trying to schedule a ChannelEmpty");
        self.do_if_present(|mut internal|{
            internal.schedule_channel_empty(&self);
            return Some(internal);
        });
        trace!("Done trying to schedule a ChannelEmpty");
    }

    fn schedule_timeout(&self, time_duration: TimeDuration)  {
        trace!("Trying to schedule a Timeout");
        self.do_if_present(|mut internal|{
            internal.schedule_timeout(&self, time_duration);
            return Some(internal);
        });
        trace!("Done trying to schedule a Timeout");
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

    fn try_receive(mut self, holder: &EventHandlerHolder<T, U>) -> Option<Self> {
        match self.receiver.try_recv_meta_data() {
            Ok((receive_meta_data, event_or_stop)) => {
                return self.on_receive(holder, receive_meta_data, event_or_stop);
            }
            Err(TryRecvError::Empty) => panic!("on_send was called when there is nothing in the channel"),
            Err(TryRecvError::Disconnected) => {
                trace!("Channel Disconnected");
                return self.on_channel_event(holder, ChannelDisconnected);
            }
        };
    }

    fn on_receive(mut self, holder: &EventHandlerHolder<T, U>, receive_meta_data: ReceiveMetaData, event_or_stop: EventOrStopThread<T::Event>) -> Option<Self> {
        match event_or_stop {
            EventOrStopThread::Event(event) => {
                trace!("Executing a ReceivedEvent");
                return self.on_channel_event(holder, ReceivedEvent(receive_meta_data, event));
            }
            EventOrStopThread::StopThread => {
                let result = self.event_handler.on_stop(receive_meta_data);
                self.join_call_back.join(AsyncJoin::new(self.thread_builder, result));
                return None;
            }
        }
    }

    fn on_channel_event(mut self, holder: &EventHandlerHolder<T, U>, event: ChannelEvent<T::Event>) -> Option<Self> {

        trace!("Event Handler: {:?}", self.thread_builder.get_name());

        match self.event_handler.on_channel_event(event) {
            Continue(WaitOrTryForNextEvent::WaitForNextEvent(event_handler)) => {
                trace!("WaitForNextEvent");
                self.event_handler = event_handler;
                return Some(self);
            }
            Continue(WaitOrTryForNextEvent::WaitForNextEventOrTimeout(event_handler, timeout_duration)) => {
                trace!("WaitForNextEventOrTimeout: {:?}", timeout_duration);
                self.event_handler = event_handler;
                self.schedule_timeout(&holder, timeout_duration);
                return Some(self);
            }
            Continue(WaitOrTryForNextEvent::TryForNextEvent(event_handler)) => {
                trace!("TryForNextEvent");
                self.event_handler = event_handler;
                self.schedule_channel_empty(&holder);
                return Some(self);
            }
            Break(result) => {
                trace!("Join");
                self.join_call_back.join(AsyncJoin::new(self.thread_builder, result));
                return None;
            }
        };
    }
}