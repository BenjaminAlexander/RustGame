use crate::interface::GameTrait;
use crate::messaging::{
    StateMessage,
};

pub trait ManagerObserverTrait: 'static + Send {
    type Game: GameTrait;

    const IS_SERVER: bool;

    fn on_step_message(&self, state_message: StateMessage<Self::Game>);

    fn on_completed_step(&self, state_message: StateMessage<Self::Game>);
}
