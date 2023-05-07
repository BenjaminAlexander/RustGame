mod enums;
mod eventhandlertrait;
mod eventhandlerthread;
mod types;
mod eventsendertrait;

pub use self::eventhandlertrait::{EventHandlerTrait};
pub use self::enums::{ChannelEvent, EventOrStopThread, WaitOrTryForNextEvent};
pub(crate) use self::eventhandlerthread::EventHandlerThread;
pub use self::types::{ChannelEventResult, SendError, SendResult, EventSender};
pub use self::eventsendertrait::EventSenderTrait;