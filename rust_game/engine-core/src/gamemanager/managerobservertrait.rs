use crate::FrameIndex;
use crate::interface::GameTrait;
use crate::messaging::{
    StateMessage,
};

//TODO: make a way for the observer to end execution on an unrecoverable error
pub trait ManagerObserverTrait: 'static + Send {
    type Game: GameTrait;

    const IS_SERVER: bool;

    fn on_input_authoritatively_missing(&self, frame_index: FrameIndex, player_index: usize);

    fn on_step_message(&self, is_state_authoritative: bool, state_message: StateMessage<Self::Game>);
}
