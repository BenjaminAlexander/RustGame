mod enums;
mod sender;
mod receiver;
mod eventhandler;
mod threadbuilder;
mod joinhandle;
mod thread;

pub(crate) use self::eventhandler::{EventHandlerTrait, EventHandlerResult};
pub(crate) use self::enums::{ChannelEvent, EventOrStopThread, WaitOrTryForNextEvent};
pub(crate) use self::joinhandle::JoinHandle;
pub(crate) use self::receiver::ReceivedEventHolder;
pub(crate) use self::sender::{EventSender, SentEventHolder, EventSendError, EventSendResult};







