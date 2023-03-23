use std::sync::mpsc;
use crate::factory::FactoryTrait;
use crate::threading::channel::SendMetaData;
use crate::threading::eventhandling::{EventOrStopThread, EventSenderTrait, SendResult};
use crate::threading::eventhandling::EventOrStopThread::{Event, StopThread};

pub type SendError<T> = mpsc::SendError<(SendMetaData, T)>;

pub trait SenderTrait<T>: Clone + Send {
    fn send(&self, factory: &impl FactoryTrait, value: T) -> Result<(), SendError<T>>;
}

impl<T, U: SenderTrait<EventOrStopThread<T>>> EventSenderTrait<T> for U {
    fn send_event(&self, factory: &impl FactoryTrait, event: T) -> SendResult<T> {
        return self.send(factory, Event(event));
    }

    fn send_stop_thread(&self, factory: &impl FactoryTrait) -> SendResult<T> {
        return self.send(factory, StopThread);
    }
}