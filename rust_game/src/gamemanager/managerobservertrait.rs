use crate::gamemanager::stepmessage::StepMessage;
use crate::interface::GameTrait;
use crate::messaging::{ServerInputMessage, StateMessage};

pub trait ManagerObserverTrait : 'static + Send {

    type Game: GameTrait;

    fn on_step_message(&self, step_message: StepMessage<Self::Game>);

    fn on_completed_step(&self, state_message: StateMessage<Self::Game>);

    fn on_server_input_message(&self, server_input_message: ServerInputMessage<Self::Game>);
}