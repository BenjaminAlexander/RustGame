use crate::threading::{
    channel::Sender,
    eventhandling::EventOrStopThread,
};

//TODO: see if this can be removed
pub type EventHandlerSender<T> = Sender<EventOrStopThread<T>>;
