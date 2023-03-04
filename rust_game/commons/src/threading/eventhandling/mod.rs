mod enums;
mod eventhandlertrait;
mod eventhandlerthread;
mod types;

pub use self::eventhandlertrait::{EventHandlerTrait};
pub use self::enums::{ChannelEvent, EventOrStopThread, WaitOrTryForNextEvent};
pub(super) use self::eventhandlerthread::EventHandlerThread;
pub use self::types::{ChannelEventResult, SendError, SendResult, Sender};