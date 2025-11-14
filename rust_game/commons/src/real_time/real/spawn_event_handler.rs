use std::ops::ControlFlow;
use std::sync::mpsc::{self, TryRecvError};
use std::thread::Builder;

use crate::real_time::ReceiveMetaData;
use crate::real_time::real::RealReceiver;
use crate::threading::eventhandling::ChannelEvent::{
    self,
    ChannelDisconnected,
    ChannelEmpty,
    ReceivedEvent,
    Timeout,
};
use crate::threading::eventhandling::EventOrStopThread::{
    Event,
    StopThread,
};
use crate::threading::eventhandling::{
    EventHandleResult,
    EventHandlerTrait,
    EventOrStopThread,
};
use crate::time::TimeDuration;
use log::info;

type EventReceiver<T> = RealReceiver<EventOrStopThread<T>>;

pub fn spawn_event_handler<T: EventHandlerTrait>(
    thread_name: String,
    receiver: EventReceiver<T::Event>,
    event_handler: T,
    call_back: impl FnOnce(T::ThreadReturn) + Send + 'static,
) -> std::io::Result<()> {
    let builder = Builder::new().name(thread_name.clone());

    builder.spawn(move || {
        info!("Thread Starting: {}", thread_name);

        let return_value = run_event_handling_loop(receiver, event_handler);

        info!(
            "Thread function complete. Invoking callback: {}",
            thread_name
        );

        (call_back)(return_value);

        info!("Thread Ending: {}", thread_name);
    })?;

    return Ok(());
}

fn run_event_handling_loop<T: EventHandlerTrait>(
    mut receiver: EventReceiver<T::Event>,
    mut message_handler: T,
) -> T::ThreadReturn {
    let mut wait_or_try = EventHandleResult::TryForNextEvent;

    loop {
        let control_flow = match wait_or_try {
            EventHandleResult::WaitForNextEvent => wait_for_message(&mut receiver),
            EventHandleResult::WaitForNextEventOrTimeout(time_duration) => {
                wait_for_message_or_timeout(&mut receiver, time_duration)
            }
            EventHandleResult::TryForNextEvent => try_for_message(&mut receiver),
            EventHandleResult::StopThread(return_value) => return return_value,
        };

        wait_or_try = match control_flow {
            ControlFlow::Continue(channel_event) => channel_event.handle(&mut message_handler),
            ControlFlow::Break(receive_meta_data) => {
                return message_handler.on_stop(receive_meta_data)
            }
        }
    }
}

fn wait_for_message_or_timeout<T: Send>(
    receiver: &mut EventReceiver<T>,
    time_duration: TimeDuration,
) -> ControlFlow<ReceiveMetaData, ChannelEvent<T>> {
    return match receiver.recv_timeout_meta_data(time_duration) {
        Ok((receive_meta_data, Event(event))) => {
            ControlFlow::Continue(ReceivedEvent(receive_meta_data, event))
        }
        Ok((receive_meta_data, StopThread)) => ControlFlow::Break(receive_meta_data),
        Err(mpsc::RecvTimeoutError::Timeout) => ControlFlow::Continue(Timeout),
        Err(mpsc::RecvTimeoutError::Disconnected) => ControlFlow::Continue(ChannelDisconnected),
    };
}

fn wait_for_message<T: Send>(
    receiver: &mut EventReceiver<T>,
) -> ControlFlow<ReceiveMetaData, ChannelEvent<T>> {
    return match receiver.recv_meta_data() {
        Ok((receive_meta_data, Event(event))) => {
            ControlFlow::Continue(ReceivedEvent(receive_meta_data, event))
        }
        Ok((receive_meta_data, StopThread)) => ControlFlow::Break(receive_meta_data),
        Err(_) => ControlFlow::Continue(ChannelDisconnected),
    };
}

fn try_for_message<T: Send>(
    receiver: &mut EventReceiver<T>,
) -> ControlFlow<ReceiveMetaData, ChannelEvent<T>> {
    return match receiver.try_recv_meta_data() {
        Ok((receive_meta_data, Event(event))) => {
            ControlFlow::Continue(ReceivedEvent(receive_meta_data, event))
        }
        Ok((receive_meta_data, StopThread)) => ControlFlow::Break(receive_meta_data),
        Err(TryRecvError::Disconnected) => ControlFlow::Continue(ChannelDisconnected),
        Err(TryRecvError::Empty) => ControlFlow::Continue(ChannelEmpty),
    };
}
