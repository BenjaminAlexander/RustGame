use std::ops::ControlFlow;
use std::sync::mpsc::{
    self,
    TryRecvError,
};
use std::thread::Builder;

use crate::real_time::event_or_stop_thread::EventOrStopThread;
use crate::real_time::real::RealReceiver;
use crate::real_time::{
    EventHandleResult,
    HandleEvent,
    ReceiveMetaData,
};
use crate::time::TimeDuration;
use log::info;

type EventReceiver<T> = RealReceiver<EventOrStopThread<T>>;

pub fn spawn_event_handler<T: HandleEvent>(
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

fn run_event_handling_loop<T: HandleEvent>(
    mut receiver: EventReceiver<T::Event>,
    mut message_handler: T,
) -> T::ThreadReturn {
    let mut wait_or_try = EventHandleResult::TryForNextEvent;

    loop {
        let control_flow = match wait_or_try {
            EventHandleResult::WaitForNextEvent => {
                wait_for_message(&mut receiver, &mut message_handler)
            }
            EventHandleResult::WaitForNextEventOrTimeout(time_duration) => {
                wait_for_message_or_timeout(&mut receiver, time_duration, &mut message_handler)
            }
            EventHandleResult::TryForNextEvent => {
                try_for_message(&mut receiver, &mut message_handler)
            }
            EventHandleResult::StopThread(return_value) => return return_value,
        };

        wait_or_try = match control_flow {
            ControlFlow::Continue(event_handle_result) => event_handle_result,
            ControlFlow::Break(receive_meta_data) => {
                return message_handler.on_stop(receive_meta_data)
            }
        }
    }
}

fn wait_for_message_or_timeout<T: HandleEvent>(
    receiver: &mut EventReceiver<T::Event>,
    time_duration: TimeDuration,
    message_handler: &mut T,
) -> ControlFlow<ReceiveMetaData, EventHandleResult<T>> {
    return match receiver.recv_timeout_meta_data(time_duration) {
        Ok((receive_meta_data, EventOrStopThread::Event(event))) => {
            ControlFlow::Continue(message_handler.on_event(receive_meta_data, event))
        }
        Ok((receive_meta_data, EventOrStopThread::StopThread)) => {
            ControlFlow::Break(receive_meta_data)
        }
        Err(mpsc::RecvTimeoutError::Timeout) => ControlFlow::Continue(message_handler.on_timeout()),
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            ControlFlow::Continue(message_handler.on_channel_disconnect())
        }
    };
}

fn wait_for_message<T: HandleEvent>(
    receiver: &mut EventReceiver<T::Event>,
    message_handler: &mut T,
) -> ControlFlow<ReceiveMetaData, EventHandleResult<T>> {
    return match receiver.recv_meta_data() {
        Ok((receive_meta_data, EventOrStopThread::Event(event))) => {
            ControlFlow::Continue(message_handler.on_event(receive_meta_data, event))
        }
        Ok((receive_meta_data, EventOrStopThread::StopThread)) => {
            ControlFlow::Break(receive_meta_data)
        }
        Err(_) => ControlFlow::Continue(message_handler.on_channel_disconnect()),
    };
}

fn try_for_message<T: HandleEvent>(
    receiver: &mut EventReceiver<T::Event>,
    message_handler: &mut T,
) -> ControlFlow<ReceiveMetaData, EventHandleResult<T>> {
    return match receiver.try_recv_meta_data() {
        Ok((receive_meta_data, EventOrStopThread::Event(event))) => {
            ControlFlow::Continue(message_handler.on_event(receive_meta_data, event))
        }
        Ok((receive_meta_data, EventOrStopThread::StopThread)) => {
            ControlFlow::Break(receive_meta_data)
        }
        Err(TryRecvError::Disconnected) => {
            ControlFlow::Continue(message_handler.on_channel_disconnect())
        }
        Err(TryRecvError::Empty) => ControlFlow::Continue(message_handler.on_channel_empty()),
    };
}
