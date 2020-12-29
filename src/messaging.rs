mod toservermessage;

use crate::simplegame::Vector2;
use serde::{Serialize, Deserialize};
use std::sync::mpsc::Sender;

pub use self::toservermessage::ToServerMessage;

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
pub struct InputMessage<InputType>
    where InputType: Clone {

    sequence: i32,
    playerIndex: i8,
    input: InputType
}

impl<InputType> InputMessage<InputType>
    where InputType: Clone {

    pub fn new(sequence: i32, playerIndex: i8, input: InputType) -> InputMessage<InputType> {
        InputMessage{ sequence, playerIndex, input }
    }
}

impl<InputType> Clone for InputMessage<InputType>
    where InputType: Clone {

    fn clone(&self) -> Self {
        Self{
            sequence: self.sequence,
            playerIndex: self.playerIndex,
            input: self.input.clone()
        }
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