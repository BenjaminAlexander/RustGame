use crate::interface::GameTrait;
use crate::messaging::{
    StateMessage,
};

/// This enum describes the provenance of the state in the [`StateMessage`]
/// TODO: rename this enum and its variants
/// TODO: or should this type be replaced with a boolean
#[derive(Debug)]
pub enum StateMessageType {
    /// The state has been computed or re-computed with incomplete information, 
    /// making it non-authoritative
    NonAuthoritativeComputed,

    /// The state has been computed or re-computed with complete information, 
    /// making it authoritative
    AuthoritativeComputed,
}

impl StateMessageType {
    //TODO: should this be removed?
    pub fn is_authoritative(&self) -> bool {
        match self {
            StateMessageType::NonAuthoritativeComputed => false,
            StateMessageType::AuthoritativeComputed => true,
        }
    }
}

pub trait ManagerObserverTrait: 'static + Send {
    type Game: GameTrait;

    const IS_SERVER: bool;

    fn on_step_message(&self, message_type: StateMessageType, state_message: StateMessage<Self::Game>);
}
