use crate::gametime::TimeMessage;
use crate::interface::GameTrait;

pub trait CoreSenderTrait<Game: GameTrait> : 'static + Send {

    fn on_time_message(&self, time_message: TimeMessage);

}