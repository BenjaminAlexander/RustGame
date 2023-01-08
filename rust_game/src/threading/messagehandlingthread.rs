use std::ops::ControlFlow;
use std::ops::ControlFlow::*;
use std::thread::JoinHandle;
use log::info;
use crate::threading::{message_channel, ValueReceiver, ValueSender, ValueTryRecvError, Thread, ThreadBuilder, ValueSendError};
use crate::threading::thread::ThreadBuilderTrait;
use crate::threading::EventOrStop::*;
use crate::threading::ChannelEvent::*;
use crate::threading::WaitOrTry::*;

pub struct SentEventHolder<T: EventHandlerTrait> {
    event: T::Event
}

pub struct ReceivedEventHolder<T: EventHandlerTrait> {
    sent_event_holder: SentEventHolder<T>
}

impl<T: EventHandlerTrait> ReceivedEventHolder<T> {

    pub fn get_event(&self) -> &T::Event { &self.sent_event_holder.event }

    pub fn move_event(self) -> T::Event { self.sent_event_holder.event }
}

pub enum ChannelEvent<T: EventHandlerTrait> {
    ReceivedEvent(ReceivedEventHolder<T>),
    ChannelEmpty,
    ChannelDisconnected
}

pub enum WaitOrTry<T: EventHandlerTrait> {
    WaitForNextEvent(T),
    TryForNextEvent(T)
}

pub enum EventOrStop<T: EventHandlerTrait> {
    Event(SentEventHolder<T>),
    StopThread
}

pub struct EventSender<T: EventHandlerTrait> {
    sender: ValueSender<EventOrStop<T>>
}

impl<T: EventHandlerTrait> EventSender<T>{

    pub fn send_event(&self, event: T::Event) -> EventSendResult<T> {
        return self.sender.send(Event(SentEventHolder { event }));
    }

    pub fn send_stop_thread(&self) -> EventSendResult<T> {
        return self.sender.send(StopThread);
    }
}

pub type EventSendError<T> = ValueSendError<EventOrStop<T>>;
pub type EventSendResult<T> = Result<(), EventSendError<T>>;
pub type EventHandlerResult<T: EventHandlerTrait> = ControlFlow<T::ThreadReturnType, WaitOrTry<T>>;

type EventReceiver<T> = ValueReceiver<EventOrStop<T>>;

pub trait EventHandlerTrait: Send + Sized + 'static {
    type Event: Send + 'static;
    type ThreadReturnType: Send + 'static;

    fn build_thread(self) -> MessageHandlingThreadBuilder<Self> {
        let (sender, receiver) = message_channel();
        return MessageHandlingThreadBuilder{
            sender: EventSender { sender },
            builder: EventHandlingThread {
                receiver,
                message_handler: self
            }.build()
        };
    }

    fn on_event(self, event: ChannelEvent<Self>) -> EventHandlerResult<Self>;

    fn on_stop(self) -> Self::ThreadReturnType;
}

//TODO: rename this
pub struct MessageHandlingThreadBuilder<MessageHandlerType: EventHandlerTrait> {
    sender: EventSender<MessageHandlerType>,
    builder: ThreadBuilder<EventHandlingThread<MessageHandlerType>>
}

impl<MessageHandlerType: EventHandlerTrait> MessageHandlingThreadBuilder<MessageHandlerType> {

    pub fn get_sender(&self) -> &EventSender<MessageHandlerType> {
        return &self.sender;
    }

}

impl<MessageHandlerType: EventHandlerTrait> ThreadBuilderTrait for MessageHandlingThreadBuilder<MessageHandlerType> {
    type StartResultType = std::io::Result<MessageHandlingThreadJoinHandle<MessageHandlerType>>;

    fn name(mut self, name: &str) -> Self {
        self.builder = self.builder.name(name);
        return self;
    }

    fn start(self) -> std::io::Result<MessageHandlingThreadJoinHandle<MessageHandlerType>> {
        let join_handle = self.builder.start()?;

        return Result::Ok(MessageHandlingThreadJoinHandle {
            sender: self.sender,
            join_handle
        });
    }
}

//TODO: rename this
pub struct MessageHandlingThreadJoinHandle<MessageHandlerType: EventHandlerTrait> {
    sender: EventSender<MessageHandlerType>,
    join_handle: JoinHandle<MessageHandlerType::ThreadReturnType>
}

impl<MessageHandlerType: EventHandlerTrait> MessageHandlingThreadJoinHandle<MessageHandlerType> {

    pub fn get_sender(&self) -> &EventSender<MessageHandlerType> {
        return &self.sender;
    }

}

struct EventHandlingThread<MessageHandlerType: EventHandlerTrait> {
    receiver: EventReceiver<MessageHandlerType>,
    message_handler: MessageHandlerType
}

impl<MessageHandlerType: EventHandlerTrait> EventHandlingThread<MessageHandlerType> {

    fn wait_for_message(message_handler: MessageHandlerType, receiver: &EventReceiver<MessageHandlerType>) -> EventHandlerResult<MessageHandlerType> {

        return match receiver.recv() {
            Ok(Event(sent_event_holder)) => Self::on_message(message_handler, sent_event_holder),
            Ok(StopThread) => Break(Self::on_stop(message_handler)),
            Err(_) => Self::on_channel_disconnected(message_handler)
        };
    }

    fn try_for_message(message_handler: MessageHandlerType, receiver: &EventReceiver<MessageHandlerType>) -> EventHandlerResult<MessageHandlerType> {

        return match receiver.try_recv() {
            Ok(Event(sent_event_holder)) => Self::on_message(message_handler, sent_event_holder),
            Ok(StopThread) => Break(Self::on_stop(message_handler)),
            Err(ValueTryRecvError::Disconnected) => Self::on_channel_disconnected(message_handler),
            Err(ValueTryRecvError::Empty) => Self::on_channel_empty(message_handler)
        };
    }

    fn on_message(message_handler: MessageHandlerType, sent_event_holder: SentEventHolder<MessageHandlerType>) -> EventHandlerResult<MessageHandlerType> {
        return message_handler.on_event(ReceivedEvent(ReceivedEventHolder { sent_event_holder }));
    }

    fn on_channel_empty(message_handler: MessageHandlerType) -> EventHandlerResult<MessageHandlerType> {
        return message_handler.on_event(ChannelEmpty);
    }

    fn on_channel_disconnected(message_handler: MessageHandlerType) -> EventHandlerResult<MessageHandlerType> {
        info!("The receiver channel has been disconnected.");
        return message_handler.on_event(ChannelDisconnected);
    }

    fn on_stop(message_handler: MessageHandlerType) -> MessageHandlerType::ThreadReturnType {
        info!("The MessageHandlingThread has received a message commanding it to stop.");
        return message_handler.on_stop();
    }
}

impl<MessageHandlerType: EventHandlerTrait> Thread for EventHandlingThread<MessageHandlerType> {
    type ReturnType = MessageHandlerType::ThreadReturnType;

    fn run(self) -> Self::ReturnType {

        let mut wait_or_try = TryForNextEvent(self.message_handler);

        loop {

            let result = match wait_or_try {
                WaitForNextEvent(message_handler) => Self::wait_for_message(message_handler, &self.receiver),
                TryForNextEvent(message_handler) => Self::try_for_message(message_handler, &self.receiver),
            };

            wait_or_try = match result {
                Continue(wait_or_try) => wait_or_try,
                Break(return_value) => {
                    return return_value;
                }
            };
        }
    }
}