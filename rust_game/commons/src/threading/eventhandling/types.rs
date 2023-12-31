use std::ops::ControlFlow;
use crate::factory::FactoryTrait;
use crate::threading::channel;
use crate::threading::eventhandling::{EventHandlerTrait, EventOrStopThread, WaitOrTryForNextEvent};

//TODO: maybe make this its own enum
pub type EventHandleResult<T> = ControlFlow<<T as EventHandlerTrait>::ThreadReturn, WaitOrTryForNextEvent<T>>;

pub type EventHandlerSendError<T> = channel::SendError<EventOrStopThread<T>>;

pub type EventHandlerSendResult<T> = Result<(), EventHandlerSendError<T>>;

pub type EventHandlerSender<Factory, T> = <Factory as FactoryTrait>::Sender<EventOrStopThread<T>>;