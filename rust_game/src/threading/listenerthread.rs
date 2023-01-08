use crate::threading::{ChannelEvent, WaitOrTry, EventHandlerTrait, EventHandlerResult};
use crate::threading::messagehandlingthread::ReceivedEventHolder;

pub enum ListenedOrDidNotListen<T: ListenerTrait> {
    Listened(T, T::ListenForType),
    DidNotListen(T)
}

pub struct ListenedValueHolder<T: ListenerTrait> {
    value: T::ListenForType
}

impl<T: ListenerTrait> ListenedValueHolder<T> {

    pub fn get_value(&self) -> &T::ListenForType {
        return &self.value;
    }

    pub fn move_value(self) -> T::ListenForType {
        return self.value;
    }
}

pub enum ListenerEvent<T: ListenerTrait> {
    ChannelEmptyAfterListen(ListenedValueHolder<T>),
    Message(ReceivedEventHolder<ListenerMessageHandler<T>>),
    ChannelDisconnected
}

pub trait ListenerTrait: Send + Sized + 'static {
    type MessageType: Send + 'static;
    type ThreadReturnType: Send + 'static;
    type ListenForType: Send + 'static;

    fn to_message_handler(self) -> ListenerMessageHandler<Self> {
        return ListenerMessageHandler::Continue(self);
    }

    fn listen(self) -> Result<ListenedOrDidNotListen<Self>, Self::ThreadReturnType>;

    fn on_event(self, event: ListenerEvent<Self>) -> Result<Self, Self::ThreadReturnType>;

    fn on_stop(self) -> Self::ThreadReturnType;
}

pub enum ListenerMessageHandler<T: ListenerTrait> {
    Heard(T, ListenedValueHolder<T>),
    Continue(T),
}

impl<T: ListenerTrait> ListenerMessageHandler<T> {

    fn listen(self) -> Result<ListenerMessageHandler<T>, T::ThreadReturnType> {
        return match
            match self {
                ListenerMessageHandler::Heard(listener, heard_value) =>
                    listener.on_event(ListenerEvent::ChannelEmptyAfterListen(heard_value))?,
                ListenerMessageHandler::Continue(listener) => listener
            }.listen()?
        {
            ListenedOrDidNotListen::Listened(listener, value) => Ok(ListenerMessageHandler::Heard(listener, ListenedValueHolder {value})),
            ListenedOrDidNotListen::DidNotListen(listener) => Ok(ListenerMessageHandler::Continue(listener))
        };
    }

    fn on_event(self, event: ListenerEvent<T>) -> Result<ListenerMessageHandler<T>, T::ThreadReturnType> {
        return match self {
            ListenerMessageHandler::Heard(listener, listened_value_holder) =>
                Ok(ListenerMessageHandler::Heard(listener.on_event(event)?, listened_value_holder)),
            ListenerMessageHandler::Continue(listener) =>
                Ok(ListenerMessageHandler::Continue(listener.on_event(event)?))
        };
    }
}

impl<T: ListenerTrait> EventHandlerTrait for ListenerMessageHandler<T> {
    type Event = T::MessageType;
    type ThreadReturnType = T::ThreadReturnType;

    fn on_event(mut self, event: ChannelEvent<Self>) -> EventHandlerResult<Self> {

        match event {
            ChannelEvent::ReceivedEvent(message) => {
                return Ok(WaitOrTry::TryForNextEvent(self.on_event(ListenerEvent::Message(message))?));
            }
            ChannelEvent::ChannelEmpty => {
                return Ok(WaitOrTry::TryForNextEvent(self.listen()?));
            }
            ChannelEvent::ChannelDisconnected => {

                self = self.on_event(ListenerEvent::ChannelDisconnected)?;

                loop {
                    self = self.listen()?;
                }
            }
        };
    }

    fn on_stop(self) -> Self::ThreadReturnType {
        match self {
            ListenerMessageHandler::Heard(listener, _) => listener.on_stop(),
            ListenerMessageHandler::Continue(listener) => listener.on_stop()
        }
    }
}