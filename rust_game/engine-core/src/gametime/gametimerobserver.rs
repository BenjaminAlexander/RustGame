use crate::gametime::TimeMessage;

//TODO: use this again to simplify game timer type parameters
pub trait GameTimerObserverTrait: 'static + Send {
    fn on_time_message(&self, time_message: TimeMessage);
}
