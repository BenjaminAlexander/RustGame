use std::ops::ControlFlow;
use crate::threading::channel;
use crate::threading::eventhandling::{EventHandlerTrait, EventOrStopThread, WaitOrTryForNextEvent};

pub(crate) type ChannelEventResult<T> = ControlFlow<<T as EventHandlerTrait>::ThreadReturn, WaitOrTryForNextEvent<T>>;

pub(crate) type JoinHandle<T> = channel::JoinHandle<EventOrStopThread<<T as EventHandlerTrait>::Event>, <T as EventHandlerTrait>::ThreadReturn>;

pub(crate) type SendError<T> = channel::SendError<EventOrStopThread<T>>;

pub(crate) type SendResult<T> = Result<(), SendError<T>>;

pub(crate) type Sender<T> = channel::Sender<EventOrStopThread<T>>;