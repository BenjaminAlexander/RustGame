use crate::gametime::TimeMessage;
use crate::interface::GameTrait;

pub trait GameTimerObserverTrait : 'static + Send {

    type Game: GameTrait;

    fn on_time_message(&self, time_message: TimeMessage);

}