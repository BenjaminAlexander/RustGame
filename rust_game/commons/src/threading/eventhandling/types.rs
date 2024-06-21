use crate::factory::FactoryTrait;
use crate::threading::channel;
use crate::threading::eventhandling::EventOrStopThread;

pub type EventHandlerSendError<T> = channel::SendError<EventOrStopThread<T>>;

pub type EventHandlerSendResult<T> = Result<(), EventHandlerSendError<T>>;

pub type EventHandlerSender<Factory, T> = <Factory as FactoryTrait>::Sender<EventOrStopThread<T>>;
