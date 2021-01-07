use serde::{Deserialize, Serialize};

pub use self::inputmessage::InputMessage;
pub use self::toservermessage::ToServerMessage;
pub use self::toclientmessage::ToClientMessage;
pub use self::statemessage::StateMessage;

mod toservermessage;
mod inputmessage;
mod toclientmessage;
mod statemessage;

#[derive(Serialize, Deserialize, Debug)]
pub struct InitialInformation<StateType>
    where StateType: Clone {

    player_count: i8,
    state_message: StateMessage<StateType>
}

impl<StateType> InitialInformation<StateType>
    where StateType: Clone {

    // pub fn new(player_index: i8, state_message: StateMessage<StateType, InputType>) -> InitialInformation<StateType, InputType> {
    //     InitialInformation{ player_index, state_message }
    // }
}

