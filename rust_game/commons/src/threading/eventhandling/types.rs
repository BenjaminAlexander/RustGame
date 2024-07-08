use crate::factory::FactoryTrait;
use crate::threading::eventhandling::EventOrStopThread;

pub type EventHandlerSender<Factory, T> = <Factory as FactoryTrait>::Sender<EventOrStopThread<T>>;
