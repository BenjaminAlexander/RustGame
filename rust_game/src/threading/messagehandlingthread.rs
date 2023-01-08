use std::thread::JoinHandle;
use log::info;
use crate::threading::{message_channel, MessageChannelReceiver, MessageChannelSender, MessageChannelTryRecvError, Thread, ThreadBuilder};
use crate::threading::thread::ThreadBuilderTrait;

pub enum MessageHandlerEvent<T: MessageHandlerTrait> {

    //TODO: refactor into struct
    Message(T::MessageType),
    ChannelEmpty,
    ChannelDisconnected
}

pub enum WaitOrTry<T: MessageHandlerTrait> {
    WaitForNextMessage(T),
    TryForNextMessage(T)
}

pub enum MessageOrStop<T: MessageHandlerTrait> {
    Message(T::MessageType),
    StopThread
}

//TODO: try using from residual instead of Result
pub type MessageHandlerEventResult<T: MessageHandlerTrait> = Result<WaitOrTry<T>, T::ThreadReturnType>;
pub type MessageHandlingThreadSender<T> = MessageChannelSender<MessageOrStop<T>>;
pub type MessageHandlingThreadReceiver<T> = MessageChannelReceiver<MessageOrStop<T>>;

pub trait MessageHandlerTrait: Send + Sized + 'static {
    type MessageType: Send + 'static;
    type ThreadReturnType: Send + 'static;

    fn build_thread(self) -> MessageHandlingThreadBuilder<Self> {
        let (sender, receiver) = message_channel();
        return MessageHandlingThreadBuilder{
            sender,
            builder: MessageHandlingThread{
                receiver,
                message_handler: self
            }.build()
        };
    }

    fn on_event(self, event: MessageHandlerEvent<Self>) -> MessageHandlerEventResult<Self>;

    fn on_stop(self) -> Self::ThreadReturnType;
}

pub struct MessageHandlingThreadBuilder<MessageHandlerType: MessageHandlerTrait> {
    sender: MessageHandlingThreadSender<MessageHandlerType>,
    builder: ThreadBuilder<MessageHandlingThread<MessageHandlerType>>
}

impl<MessageHandlerType: MessageHandlerTrait> MessageHandlingThreadBuilder<MessageHandlerType> {

    pub fn get_sender(&self) -> &MessageHandlingThreadSender<MessageHandlerType> {
        return &self.sender;
    }

}

impl<MessageHandlerType: MessageHandlerTrait> ThreadBuilderTrait for MessageHandlingThreadBuilder<MessageHandlerType> {
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

pub struct MessageHandlingThreadJoinHandle<MessageHandlerType: MessageHandlerTrait> {
    sender: MessageHandlingThreadSender<MessageHandlerType>,
    join_handle: JoinHandle<MessageHandlerType::ThreadReturnType>
}

impl<MessageHandlerType: MessageHandlerTrait> MessageHandlingThreadJoinHandle<MessageHandlerType> {

    pub fn get_sender(&self) -> &MessageHandlingThreadSender<MessageHandlerType> {
        return &self.sender;
    }

}

struct MessageHandlingThread<MessageHandlerType: MessageHandlerTrait> {
    receiver: MessageHandlingThreadReceiver<MessageHandlerType>,
    message_handler: MessageHandlerType
}

impl<MessageHandlerType: MessageHandlerTrait> MessageHandlingThread<MessageHandlerType> {

    fn wait_for_message(message_handler: MessageHandlerType, receiver: &MessageHandlingThreadReceiver<MessageHandlerType>) -> MessageHandlerEventResult<MessageHandlerType> {

        return match receiver.recv() {
            Ok(MessageOrStop::Message(message)) => Self::on_message(message_handler, message),
            Ok(MessageOrStop::StopThread) => Err(Self::on_stop(message_handler)),
            Err(_) => Self::on_channel_disconnected(message_handler)
        };
    }

    fn try_for_message(message_handler: MessageHandlerType, receiver: &MessageHandlingThreadReceiver<MessageHandlerType>) -> MessageHandlerEventResult<MessageHandlerType> {

        return match receiver.try_recv() {
            Ok(MessageOrStop::Message(message)) => Self::on_message(message_handler, message),
            Ok(MessageOrStop::StopThread) => Err(Self::on_stop(message_handler)),
            Err(MessageChannelTryRecvError::Disconnected) => Self::on_channel_disconnected(message_handler),
            Err(MessageChannelTryRecvError::Empty) => Self::on_channel_empty(message_handler)
        };
    }

    fn on_message(message_handler: MessageHandlerType, message: MessageHandlerType::MessageType) -> MessageHandlerEventResult<MessageHandlerType> {
        return message_handler.on_event(MessageHandlerEvent::Message(message));
    }

    fn on_channel_empty(message_handler: MessageHandlerType) -> MessageHandlerEventResult<MessageHandlerType> {
        return message_handler.on_event(MessageHandlerEvent::ChannelEmpty);
    }

    fn on_channel_disconnected(message_handler: MessageHandlerType) -> MessageHandlerEventResult<MessageHandlerType> {
        info!("The receiver channel has been disconnected.");
        return message_handler.on_event(MessageHandlerEvent::ChannelDisconnected);
    }

    fn on_stop(message_handler: MessageHandlerType) -> MessageHandlerType::ThreadReturnType {
        info!("The MessageHandlingThread has received a message commanding it to stop.");
        return message_handler.on_stop();
    }
}

impl<MessageHandlerType: MessageHandlerTrait> Thread for MessageHandlingThread<MessageHandlerType> {
    type ReturnType = MessageHandlerType::ThreadReturnType;

    fn run(self) -> Self::ReturnType {

        info!("Thread Starting");

        let mut wait_or_try = WaitOrTry::TryForNextMessage(self.message_handler);

        loop {

            let result = match wait_or_try {
                WaitOrTry::WaitForNextMessage(message_handler) => Self::wait_for_message(message_handler, &self.receiver),
                WaitOrTry::TryForNextMessage(message_handler) => Self::try_for_message(message_handler, &self.receiver),
            };

            wait_or_try = match result {
                Ok(wait_or_try) => wait_or_try,
                Err(return_value) => {
                    return return_value;
                }
            };
        }
    }
}

/*
                WaitOrTry::Stop(thread_return) => {
                    info!("The MessageHandler commanded the thread to stop.");
                    return thread_return;
                }
 */