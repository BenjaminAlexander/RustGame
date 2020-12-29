mod toservermessage;
mod inputmessage;

use crate::simplegame::Vector2;
use serde::{Serialize, Deserialize};
use std::sync::mpsc::Sender;

pub use self::toservermessage::ToServerMessage;
pub use self::inputmessage::InputMessage;

#[derive(Serialize, Deserialize, Debug)]
pub struct InitialInformation<StateType, InputType> {
    playerIndex: i8,
    stateMessage: StateMessage<StateType, InputType>
}

impl<StateType, InputType> InitialInformation<StateType, InputType> {
    pub fn new(playerIndex: i8,  stateMessage: StateMessage<StateType, InputType>) -> InitialInformation<StateType, InputType> {
        InitialInformation{ playerIndex, stateMessage }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StateMessage<StateType, InputType> {
    sequence: i32,
    inputs: Vec<Option<InputType>>,
    state: Option<StateType>
}

impl<StateType, InputType> StateMessage<StateType, InputType> {
    pub fn new(sequence: i32, inputs: Vec<Option<InputType>>, state: Option<StateType>) -> StateMessage<StateType, InputType> {
        StateMessage{ sequence, inputs, state }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TimeSyncMessage {
    scheduledTime: i32,
    actualTime: i32,
    sequence: i32,
}

impl TimeSyncMessage {
    pub fn new(scheduledTime: i32, actualTime: i32, sequence: i32) -> TimeSyncMessage {
        TimeSyncMessage{ scheduledTime, actualTime, sequence }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Message<StateType, InputType>
    where InputType: Clone {

    Input(InputMessage<InputType>),
    State(StateMessage<StateType, InputType>)
}