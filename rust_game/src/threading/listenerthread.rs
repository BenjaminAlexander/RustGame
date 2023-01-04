use crate::threading::{MessageHandlerEvent, MessageHandlerThreadAction, MessageHandlerTrait};

pub enum AfterListen<T: ListenerTrait> {
    Heard(T, T::ListenForType),
    Continue(T),
    Stop(T::ThreadReturnType)
}

impl<T: ListenerTrait> AfterListen<T> {

    fn to_listener_message_handler_action(self) -> ListenerMessageHandler<T> {
        match self {
            Self::Heard(listener, value) =>
                ListenerMessageHandler::Heard(listener, HeardValue{value}),
            Self::Continue(listener) => ListenerMessageHandler::Continue(listener),
            Self::Stop(return_value) => ListenerMessageHandler::Stop(return_value)
        }
    }
}

pub enum ListenerAction<T: ListenerTrait> {
    Continue(T),
    Stop(T::ThreadReturnType)
}

impl<T: ListenerTrait> ListenerAction<T> {

    fn to_listener_message_handler_action(self) -> ListenerMessageHandler<T> {
        match self {
            Self::Continue(listener) => ListenerMessageHandler::Continue(listener),
            Self::Stop(return_value) => ListenerMessageHandler::Stop(return_value)
        }
    }
}

pub struct HeardValue<T: ListenerTrait> {
    value: T::ListenForType
}

impl<T: ListenerTrait> HeardValue<T> {

    pub fn get_value(&self) -> &T::ListenForType {
        return &self.value;
    }

    pub fn move_value(self) -> T::ListenForType {
        return self.value;
    }
}

pub enum ListenerEvent<T: ListenerTrait> {
    Heard(HeardValue<T>),
    Message(T::MessageType),
    ChannelDisconnected
}

pub trait ListenerTrait: Send + Sized + 'static {
    type MessageType: Send + 'static;
    type ThreadReturnType: Send + 'static;
    type ListenForType: Send + 'static;

    fn to_message_handler(self) -> ListenerMessageHandler<Self> {
        return ListenerMessageHandler::Continue(self);
    }

    fn listen(self) -> AfterListen<Self>;

    fn on_event(self, event: ListenerEvent<Self>) -> ListenerAction<Self>;

    fn on_stop(self) -> Self::ThreadReturnType;
}

pub enum ListenerMessageHandler<T: ListenerTrait> {
    Heard(T, HeardValue<T>),
    Continue(T),
    Stop(T::ThreadReturnType)
}

impl<T: ListenerTrait> ListenerMessageHandler<T> {

    fn listen(self) -> ListenerMessageHandler<T> {
        match self {
            ListenerMessageHandler::Heard(listener, heard_value) =>
                Self::listen(listener.on_event(ListenerEvent::Heard(heard_value)).to_listener_message_handler_action()),
            ListenerMessageHandler::Continue(listener) =>
                listener.listen().to_listener_message_handler_action(),
            ListenerMessageHandler::Stop(return_value) =>
                ListenerMessageHandler::Stop(return_value)
        }
    }

    fn on_message(self, message: T::MessageType) -> ListenerMessageHandler<T> {
        match self {
            ListenerMessageHandler::Heard(listener, heard_value) =>
                match listener.on_event(ListenerEvent::Message(message)) {
                    ListenerAction::Continue(listener) => ListenerMessageHandler::Heard(listener, heard_value),
                    ListenerAction::Stop(return_value) => ListenerMessageHandler::Stop(return_value),
                },
            ListenerMessageHandler::Continue(listener) =>
                listener.on_event(ListenerEvent::Message(message)).to_listener_message_handler_action(),
            ListenerMessageHandler::Stop(return_value) =>
                ListenerMessageHandler::Stop(return_value)
        }
    }

    fn on_channel_disconnected(self) -> ListenerMessageHandler<T> {
        match self {
            ListenerMessageHandler::Heard(listener, heard_value) =>
                match listener.on_event(ListenerEvent::ChannelDisconnected) {
                    ListenerAction::Continue(listener) => ListenerMessageHandler::Heard(listener, heard_value),
                    ListenerAction::Stop(return_value) => ListenerMessageHandler::Stop(return_value),
                },
            ListenerMessageHandler::Continue(listener) =>
                listener.on_event(ListenerEvent::ChannelDisconnected).to_listener_message_handler_action(),
            ListenerMessageHandler::Stop(return_value) =>
                ListenerMessageHandler::Stop(return_value)
        }
    }

    fn to_message_handler_thread_action(self) -> MessageHandlerThreadAction<Self> {
        match self {
            ListenerMessageHandler::Heard(listener, heard_value) =>
                MessageHandlerThreadAction::TryForNextMessage(ListenerMessageHandler::Heard(listener, heard_value)),
            ListenerMessageHandler::Continue(listener) =>
                MessageHandlerThreadAction::TryForNextMessage(ListenerMessageHandler::Continue(listener)),
            ListenerMessageHandler::Stop(return_value) =>
                MessageHandlerThreadAction::Stop(return_value)
        }
    }
}

impl<T: ListenerTrait> MessageHandlerTrait for ListenerMessageHandler<T> {
    type MessageType = T::MessageType;
    type ThreadReturnType = T::ThreadReturnType;

    fn on_event(mut self, event: MessageHandlerEvent<Self>) -> MessageHandlerThreadAction<Self> {
        match event {
            MessageHandlerEvent::Message(message) => {
                return self.on_message(message).to_message_handler_thread_action();
            }
            MessageHandlerEvent::ChannelEmpty => {
                return self.listen().to_message_handler_thread_action();
            }
            MessageHandlerEvent::ChannelDisconnected => {
                self = self.on_channel_disconnected();

                loop {
                    match self {
                        ListenerMessageHandler::Heard(listener, heard_value) => {
                            self = ListenerMessageHandler::Heard(listener, heard_value).listen();
                        }
                        ListenerMessageHandler::Continue(listener) => {
                            self = ListenerMessageHandler::Continue(listener).listen();
                        }
                        ListenerMessageHandler::Stop(return_value) => {
                            return MessageHandlerThreadAction::Stop(return_value);
                        }
                    }
                }
            }
        }
    }

    fn on_stop(self) -> Self::ThreadReturnType {
        match self {
            ListenerMessageHandler::Heard(listener, _) => listener.on_stop(),
            ListenerMessageHandler::Continue(listener) => listener.on_stop(),
            ListenerMessageHandler::Stop(return_value) => return_value
        }
    }
}