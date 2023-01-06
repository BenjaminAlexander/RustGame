use crate::threading::{MessageHandlerEvent, MessageHandlerThreadAction, MessageHandlerTrait};

pub enum ListenResult<T: ListenerTrait> {
    Listened(T, T::ListenForType),
    DidNotListen(ListenerEventResult<T>)
}

impl<T: ListenerTrait> ListenResult<T> {

    //TODO: rename this
    fn to_listener_message_handler_action(self) -> MessageHandlerThreadAction<ListenerMessageHandler<T>> {
        match self {
            Self::Listened(listener, value) =>
                MessageHandlerThreadAction::TryForNextMessage(ListenerMessageHandler::Heard(listener, ListenedValue {value})),
            Self::DidNotListen(ListenerEventResult::Continue(listener)) =>
                MessageHandlerThreadAction::TryForNextMessage(ListenerMessageHandler::Continue(listener)),
            Self::DidNotListen(ListenerEventResult::Stop(return_value)) =>
                MessageHandlerThreadAction::Stop(return_value)
        }
    }
}

pub enum ListenerEventResult<T: ListenerTrait> {
    Continue(T),
    Stop(T::ThreadReturnType)
}

impl<T: ListenerTrait> ListenerEventResult<T> {

    fn to_listener_message_handler_action(self) -> ListenerMessageHandler<T> {
        match self {
            Self::Continue(listener) => ListenerMessageHandler::Continue(listener),
            Self::Stop(return_value) => ListenerMessageHandler::Stop(return_value)
        }
    }
}

pub struct ListenedValue<T: ListenerTrait> {
    value: T::ListenForType
}

impl<T: ListenerTrait> ListenedValue<T> {

    pub fn get_value(&self) -> &T::ListenForType {
        return &self.value;
    }

    pub fn move_value(self) -> T::ListenForType {
        return self.value;
    }
}

pub enum ListenerEvent<T: ListenerTrait> {
    ChannelEmptyAfterListen(ListenedValue<T>),
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

    fn listen(self) -> ListenResult<Self>;

    fn on_event(self, event: ListenerEvent<Self>) -> ListenerEventResult<Self>;

    fn on_stop(self) -> Self::ThreadReturnType;
}

pub enum ListenerMessageHandler<T: ListenerTrait> {
    Heard(T, ListenedValue<T>),
    Continue(T),

    //TODO: can this be removed
    Stop(T::ThreadReturnType)
}

impl<T: ListenerTrait> ListenerMessageHandler<T> {

    fn listen(self) -> MessageHandlerThreadAction<ListenerMessageHandler<T>> {
        match self {
            ListenerMessageHandler::Heard(listener, heard_value) =>
                match listener.on_event(ListenerEvent::ChannelEmptyAfterListen(heard_value)) {
                    ListenerEventResult::Continue(listener) =>
                        ListenerMessageHandler::Continue(listener).listen(),
                    ListenerEventResult::Stop(return_value) =>
                        MessageHandlerThreadAction::Stop(return_value)
                }
            ListenerMessageHandler::Continue(listener) =>
                listener.listen().to_listener_message_handler_action(),
            ListenerMessageHandler::Stop(return_value) =>
                MessageHandlerThreadAction::Stop(return_value)
        }
    }

    fn on_message(self, message: T::MessageType) -> MessageHandlerThreadAction<ListenerMessageHandler<T>> {
        match self {
            ListenerMessageHandler::Heard(listener, heard_value) =>
                match listener.on_event(ListenerEvent::Message(message)) {
                    ListenerEventResult::Continue(listener) => ListenerMessageHandler::Heard(listener, heard_value),
                    ListenerEventResult::Stop(return_value) => ListenerMessageHandler::Stop(return_value),
                },
            ListenerMessageHandler::Continue(listener) =>
                listener.on_event(ListenerEvent::Message(message)).to_listener_message_handler_action(),
            ListenerMessageHandler::Stop(return_value) =>
                ListenerMessageHandler::Stop(return_value)
        }.to_message_handler_thread_action()
    }

    fn on_channel_disconnected(self) -> ListenerMessageHandler<T> {
        match self {
            ListenerMessageHandler::Heard(listener, heard_value) =>
                match listener.on_event(ListenerEvent::ChannelDisconnected) {
                    ListenerEventResult::Continue(listener) => ListenerMessageHandler::Heard(listener, heard_value),
                    ListenerEventResult::Stop(return_value) => ListenerMessageHandler::Stop(return_value),
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
                return self.on_message(message);
            }
            MessageHandlerEvent::ChannelEmpty => {
                return self.listen();
            }
            MessageHandlerEvent::ChannelDisconnected => {

                self = match self {
                    ListenerMessageHandler::Heard(listener, listened_value) => {
                        match listener.on_event(ListenerEvent::ChannelDisconnected) {
                            ListenerEventResult::Continue(listener) => ListenerMessageHandler::Heard(listener, listened_value),
                            ListenerEventResult::Stop(return_value) => {
                                return MessageHandlerThreadAction::Stop(return_value);
                            }
                        }
                    }
                    ListenerMessageHandler::Continue(listener) => {
                        match listener.on_event(ListenerEvent::ChannelDisconnected) {
                            ListenerEventResult::Continue(listener) => ListenerMessageHandler::Continue(listener),
                            ListenerEventResult::Stop(return_value) => {
                                return MessageHandlerThreadAction::Stop(return_value);
                            }
                        }
                    }
                    ListenerMessageHandler::Stop(return_value) => {
                        return MessageHandlerThreadAction::Stop(return_value);
                    }
                };

                loop {
                    self = match self {
                        ListenerMessageHandler::Heard(listener, heard_value) => {
                            match listener.on_event(ListenerEvent::ChannelEmptyAfterListen(heard_value)) {
                                ListenerEventResult::Continue(listener) => ListenerMessageHandler::Continue(listener),
                                ListenerEventResult::Stop(return_value) => {
                                    return MessageHandlerThreadAction::Stop(return_value);
                                }
                            }
                        }
                        ListenerMessageHandler::Continue(listener) => {
                            match listener.listen() {
                                ListenResult::Listened(listener, listened_value) => ListenerMessageHandler::Heard(listener, ListenedValue{value: listened_value}),
                                ListenResult::DidNotListen(ListenerEventResult::Continue(listener)) => ListenerMessageHandler::Continue(listener),
                                ListenResult::DidNotListen(ListenerEventResult::Stop(return_value)) => {
                                    return MessageHandlerThreadAction::Stop(return_value);
                                }
                            }
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