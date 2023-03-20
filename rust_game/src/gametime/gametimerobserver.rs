use crate::gametime::TimeMessage;

pub trait GameTimerObserverTrait : 'static + Send {

    fn on_time_message(&self, time_message: TimeMessage);

}