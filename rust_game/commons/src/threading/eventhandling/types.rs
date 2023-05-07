use std::ops::ControlFlow;
use crate::factory::FactoryTrait;
use crate::threading::channel;
use crate::threading::eventhandling::{EventHandlerTrait, EventOrStopThread, WaitOrTryForNextEvent};

pub type ChannelEventResult<T> = ControlFlow<<T as EventHandlerTrait>::ThreadReturn, WaitOrTryForNextEvent<T>>;

//TODO: rename
pub type SendError<T> = channel::SendError<EventOrStopThread<T>>;

//TODO: rename
pub type SendResult<T> = Result<(), SendError<T>>;

pub type EventSender<Factory, T> = <Factory as FactoryTrait>::Sender<EventOrStopThread<T>>;