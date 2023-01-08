mod enums;
mod sender;
mod receivedeventholder;
mod eventhandlertrait;
mod threadbuilder;
mod joinhandle;
mod thread;

pub(crate) use self::eventhandlertrait::{EventHandlerTrait, ChannelEventResult, build_thread};
pub(crate) use self::enums::{ChannelEvent, EventOrStopThread, WaitOrTryForNextEvent};
pub(crate) use self::joinhandle::JoinHandle;
pub(crate) use self::receivedeventholder::ReceivedEventHolder;
pub(crate) use self::sender::{Sender, SentEventHolder, SendError, SendResult};







