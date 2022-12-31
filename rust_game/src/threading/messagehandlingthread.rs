use std::thread::JoinHandle;
use log::info;
use crate::threading::{message_channel, MessageChannelReceiver, MessageChannelSender, MessageChannelTryRecvError, Thread, ThreadBuilder};
use crate::threading::thread::ThreadBuilderTrait;

pub enum ContinueOrStop<T: MessageHandlerTrait> {
    Continue(T),
    Stop(T::ThreadReturnType)
}

pub enum MessageOrStop<T: MessageHandlerTrait> {
    Message(T::MessageType),
    Stop
}

pub type MessageHandlingThreadSender<T> = MessageChannelSender<MessageOrStop<T>>;
pub type MessageHandlingThreadReceiver<T> = MessageChannelReceiver<MessageOrStop<T>>;

pub trait MessageHandlerTrait: Send + Sized + 'static {
    type MessageType: Send + 'static;
    type ThreadReturnType: Send + 'static;

    fn build(self) -> MessageHandlingThreadBuilder<Self> {
        let (sender, receiver) = message_channel();
        return MessageHandlingThreadBuilder{
            sender,
            builder: MessageHandlingThread{
                receiver,
                message_handler: self
            }.build()
        };
    }

    fn on_message(self, message: Self::MessageType) -> ContinueOrStop<Self>;

    fn on_none_pending(self) -> ContinueOrStop<Self>;

    fn on_channel_disconnect(self) -> ContinueOrStop<Self>;

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

impl<MessageHandlerType: MessageHandlerTrait> Thread for MessageHandlingThread<MessageHandlerType> {
    type ReturnType = MessageHandlerType::ThreadReturnType;

    fn run(mut self) -> Self::ReturnType {

        info!("Thread Starting");

        loop {

            //Wait for and handle the first message
            match self.receiver.recv() {
                Ok(MessageOrStop::Message(message)) => {
                    match self.message_handler.on_message(message) {
                        ContinueOrStop::Continue(next_self) => { self.message_handler = next_self; }
                        ContinueOrStop::Stop(return_value) => {
                            info!("After handling a message, the MessageHandler commanded the thread to stop.");
                            return return_value;
                        }
                    }
                }
                Ok(MessageOrStop::Stop) => { return self.message_handler.on_stop(); }
                Err(_) => {
                    match self.message_handler.on_channel_disconnect() {
                        ContinueOrStop::Continue(next_self) => { self.message_handler = next_self; }
                        ContinueOrStop::Stop(return_value) => {
                            info!("After channel disconnect, the MessageHandler commanded the thread to stop.");
                            return return_value;
                        }
                    }
                }
            }

            //Handle the rest of the messages in the queue
            loop {
                match self.receiver.try_recv() {
                    Ok(MessageOrStop::Message(message)) => {
                        match self.message_handler.on_message(message) {
                            ContinueOrStop::Continue(next_self) => { self.message_handler = next_self; }
                            ContinueOrStop::Stop(return_value) => {
                                info!("After handling a message, the MessageHandler commanded the thread to stop.");
                                return return_value;
                            }
                        }
                    }
                    Ok(MessageOrStop::Stop) => { return self.message_handler.on_stop(); }
                    Err(MessageChannelTryRecvError::Disconnected) => {
                        match self.message_handler.on_channel_disconnect() {
                            ContinueOrStop::Continue(next_self) => { self.message_handler = next_self; }
                            ContinueOrStop::Stop(return_value) => {
                                info!("After channel disconnect, the MessageHandler commanded the thread to stop.");
                                return return_value;
                            }
                        }
                    }
                    Err(MessageChannelTryRecvError::Empty) => {
                        match self.message_handler.on_none_pending() {
                            ContinueOrStop::Continue(next_self) => {
                                self.message_handler = next_self;
                                continue;
                            }
                            ContinueOrStop::Stop(return_value) => {
                                info!("After no messages pending, the MessageHandler commanded the thread to stop.");
                                return return_value;
                            }
                        }
                    }
                }
            }
        }
    }
}