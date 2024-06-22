use crate::factory::FactoryTrait;
use crate::threading::channel;
use crate::threading::eventhandling::EventOrStopThread;

pub type EventHandlerSendResult<T> = Result<(), EventOrStopThread<T>>;

pub type EventHandlerSender<Factory, T> = <Factory as FactoryTrait>::Sender<EventOrStopThread<T>>;
