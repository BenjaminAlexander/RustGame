use crate::interface::GameTrait;
use crate::messaging::{
    StateMessage,
};

pub trait ManagerObserverTrait: 'static + Send {
    type Game: GameTrait;

    const IS_SERVER: bool;

    fn on_step_message(&self, is_state_authoritative: bool, state_message: StateMessage<Self::Game>);
}
