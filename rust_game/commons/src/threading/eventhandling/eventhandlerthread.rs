use std::sync::mpsc::TryRecvError;
use std::thread::Builder;

use crate::threading::channel::{
    RealReceiver,
    ReceiveMetaData,
    ReceiverTrait,
    RecvTimeoutError,
};
use crate::threading::eventhandling::ChannelEvent::{
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

//TODO: remove
type EventReceiver<T> = RealReceiver<EventOrStopThread<T>>;

//TODO: is this type coupled to RealReceiver
pub struct EventHandlerThread<T: EventHandlerTrait> {
    receiver: EventReceiver<T::Event>,
    event_handler: T,
}

impl<T: EventHandlerTrait> EventHandlerThread<T> {
    pub(crate) fn spawn_thread(
        thread_name: String,
        receiver: EventReceiver<T::Event>,
        event_handler: T,
        call_back: impl FnOnce(T::ThreadReturn) + Send + 'static,
    ) -> std::io::Result<()> {
        let thread = Self {
            receiver,
            event_handler,
        };

        let builder = Builder::new().name(thread_name.clone());

        builder.spawn(move || {
            info!("Thread Starting: {}", thread_name);

            let return_value = thread.run();

            info!(
                "Thread function complete. Invoking callback: {}",
                thread_name
            );

            (call_back)(return_value);

            info!("Thread Ending: {}", thread_name);
        })?;

        return Ok(());
    }

    fn wait_for_message_or_timeout(
        message_handler: T,
        receiver: &mut EventReceiver<T::Event>,
        time_duration: TimeDuration,
    ) -> EventHandleResult<T> {
        return match receiver.recv_timeout_meta_data(time_duration) {
            Ok((receive_meta_data, Event(event))) => {
                Self::on_message(message_handler, receive_meta_data, event)
            }
            Ok((receive_meta_data, StopThread)) => {
                EventHandleResult::StopThread(Self::on_stop(message_handler, receive_meta_data))
            }
            Err(RecvTimeoutError::Timeout) => Self::on_timeout(message_handler),
            Err(RecvTimeoutError::Disconnected) => Self::on_channel_disconnected(message_handler),
        };
    }

    fn wait_for_message(
        message_handler: T,
        receiver: &mut EventReceiver<T::Event>,
    ) -> EventHandleResult<T> {
        return match receiver.recv_meta_data() {
            Ok((receive_meta_data, Event(event))) => {
                Self::on_message(message_handler, receive_meta_data, event)
            }
            Ok((receive_meta_data, StopThread)) => {
                EventHandleResult::StopThread(Self::on_stop(message_handler, receive_meta_data))
            }
            Err(_) => Self::on_channel_disconnected(message_handler),
        };
    }

    fn try_for_message(
        message_handler: T,
        receiver: &mut EventReceiver<T::Event>,
    ) -> EventHandleResult<T> {
        return match receiver.try_recv_meta_data() {
            Ok((receive_meta_data, Event(event))) => {
                Self::on_message(message_handler, receive_meta_data, event)
            }
            Ok((receive_meta_data, StopThread)) => {
                EventHandleResult::StopThread(Self::on_stop(message_handler, receive_meta_data))
            }
            Err(TryRecvError::Disconnected) => Self::on_channel_disconnected(message_handler),
            Err(TryRecvError::Empty) => Self::on_channel_empty(message_handler),
        };
    }

    fn on_message(
        message_handler: T,
        receive_meta_data: ReceiveMetaData,
        event: T::Event,
    ) -> EventHandleResult<T> {
        return message_handler.on_channel_event(ReceivedEvent(receive_meta_data, event));
    }

    fn on_channel_empty(message_handler: T) -> EventHandleResult<T> {
        return message_handler.on_channel_event(ChannelEmpty);
    }

    fn on_timeout(message_handler: T) -> EventHandleResult<T> {
        return message_handler.on_channel_event(Timeout);
    }

    fn on_channel_disconnected(message_handler: T) -> EventHandleResult<T> {
        info!("The receiver channel has been disconnected.");
        return message_handler.on_channel_event(ChannelDisconnected);
    }

    fn on_stop(message_handler: T, receive_meta_data: ReceiveMetaData) -> T::ThreadReturn {
        info!("The MessageHandlingThread has received a message commanding it to stop.");
        return message_handler.on_stop(receive_meta_data);
    }

    pub(crate) fn run(mut self) -> T::ThreadReturn {
        let mut wait_or_try = EventHandleResult::TryForNextEvent(self.event_handler);

        loop {
            wait_or_try = match wait_or_try {
                EventHandleResult::WaitForNextEvent(message_handler) => {
                    Self::wait_for_message(message_handler, &mut self.receiver)
                }
                EventHandleResult::WaitForNextEventOrTimeout(message_handler, time_duration) => {
                    Self::wait_for_message_or_timeout(
                        message_handler,
                        &mut self.receiver,
                        time_duration,
                    )
                }
                EventHandleResult::TryForNextEvent(message_handler) => {
                    Self::try_for_message(message_handler, &mut self.receiver)
                }
                EventHandleResult::StopThread(return_value) => {
                    return return_value;
                }
            };
        }
    }
}
