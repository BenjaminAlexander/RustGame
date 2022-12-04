use crate::gametime::TimeMessage;
use crate::interface::GameTrait;
use crate::messaging::StateMessage;

pub trait CoreSenderTrait : 'static + Send {

    type Game: GameTrait;

    fn on_time_message(&self, time_message: TimeMessage);

    fn on_completed_step(&self, state_message: StateMessage<Self::Game>);

}