use crate::interface::GameTrait;
use crate::messaging::{
    StateMessage,
};

/// This enum describes the provenance of the state in the [`StateMessage`]
pub enum StateMessageType {
    /// The state has been computed or re-computed with incomplete information, 
    /// making it non-authoritative
    NonAuthoritativeComputed,

    /// The state has been computed or re-computed with complete information, 
    /// making it authoritative
    AuthoritativeComputed,

    /// The state has become authoritative because time has expired for clients 
    /// to submit inputs.  It has not been recomputed since the last time this 
    /// state was contained in a message.
    AuthoritativeTimeout,
}

impl StateMessageType {
    pub fn is_authoritative(&self) -> bool {
        match self {
            StateMessageType::NonAuthoritativeComputed => false,
            StateMessageType::AuthoritativeComputed => true,
            StateMessageType::AuthoritativeTimeout => true,
        }
    }

    pub fn is_changed(&self) -> bool {
        match self {
            StateMessageType::NonAuthoritativeComputed => false,
            StateMessageType::AuthoritativeComputed => false,
            StateMessageType::AuthoritativeTimeout => true,
        }
    }
}

pub trait ManagerObserverTrait: 'static + Send {
    type Game: GameTrait;

    const IS_SERVER: bool;

    fn on_step_message(&self, message_type: StateMessageType, state_message: StateMessage<Self::Game>);
}
