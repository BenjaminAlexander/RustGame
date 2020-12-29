mod toservermessage;
mod inputmessage;

use serde::{Serialize, Deserialize};
pub use self::toservermessage::ToServerMessage;
pub use self::inputmessage::InputMessage;

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

#[derive(Serialize, Deserialize, Debug)]
pub struct TimeSyncMessage {
    scheduled_time: i32,
    actual_time: i32,
    sequence: i32,
}

impl TimeSyncMessage {
    // pub fn new(scheduled_time: i32, actual_time: i32, sequence: i32) -> TimeSyncMessage {
    //     TimeSyncMessage{ scheduled_time, actual_time, sequence }
    // }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Message<StateType, InputType>
    where InputType: Clone {

    Input(InputMessage<InputType>),
    State(StateMessage<StateType, InputType>)
}