use crate::threading::channel::ChannelThreadJoinHandle;
use crate::threading::eventhandling::EventOrStopThread;

//TODO: don't reference EventHandlerTrait
//TODO: move this to channel
//TODO: make event handling type alias
pub type JoinHandle<T, U> = ChannelThreadJoinHandle<EventOrStopThread<T>, U>;