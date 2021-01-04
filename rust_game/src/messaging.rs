use serde::{Deserialize, Serialize};

pub use self::inputmessage::InputMessage;
pub use self::toservermessage::ToServerMessage;
pub use self::toclientmessage::ToClientMessage;

mod toservermessage;
mod inputmessage;
mod toclientmessage;

#[derive(Serialize, Deserialize, Debug)]
pub struct InitialInformation<StateType, InputType> {
    player_index: i8,
    state_message: StateMessage<StateType, InputType>
}

impl<StateType, InputType> InitialInformation<StateType, InputType> {
    // pub fn new(player_index: i8, state_message: StateMessage<StateType, InputType>) -> InitialInformation<StateType, InputType> {
    //     InitialInformation{ player_index, state_message }
    // }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StateMessage<StateType, InputType> {
    sequence: i32,
    inputs: Vec<Option<InputType>>,
    state: Option<StateType>
}

impl<StateType, InputType> StateMessage<StateType, InputType> {
    // pub fn new(sequence: i32, inputs: Vec<Option<InputType>>, state: Option<StateType>) -> StateMessage<StateType, InputType> {
    //     StateMessage{ sequence, inputs, state }
    // }
}