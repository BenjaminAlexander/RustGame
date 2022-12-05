use crate::interface::GameTrait;
use crate::messaging::StateMessage;

pub trait CoreSenderTrait : 'static + Send {

    type Game: GameTrait;

    fn on_completed_step(&self, state_message: StateMessage<Self::Game>);

}