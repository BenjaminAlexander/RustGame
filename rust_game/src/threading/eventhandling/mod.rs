mod enums;
mod eventsendertrait;
mod eventhandlertrait;
mod threadbuilder;
mod joinhandle;
mod thread;

use crate::threading::channel;

pub(crate) use self::eventhandlertrait::{EventHandlerTrait, ChannelEventResult, build_thread};
pub(crate) use self::enums::{ChannelEvent, EventOrStopThread, WaitOrTryForNextEvent};
pub(crate) use self::joinhandle::JoinHandle;
pub(crate) use self::eventsendertrait::EventSenderTrait;
pub(crate) use self::threadbuilder::{ThreadBuilder, EventHandlerThreadBuilderTrait, EventHandlerChannelThreadBuilderTrait};
pub(in crate::threading) use self::thread::Thread;

pub(crate) type SendError<T> = channel::SendError<EventOrStopThread<T>>;

pub(crate) type SendResult<T> = Result<(), SendError<T>>;

pub(crate) type Sender<T> = channel::Sender<EventOrStopThread<T>>;







