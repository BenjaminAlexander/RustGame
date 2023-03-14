use crate::gametime::TimeMessage;
use crate::interface::GameTrait;

pub trait GameTimerObserverTrait : 'static + Send {

    fn on_time_message(&self, time_message: TimeMessage);

}