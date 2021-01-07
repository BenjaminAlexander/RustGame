use serde::{Deserialize, Serialize};
use crate::messaging::InputMessage;

#[derive(Serialize, Deserialize, Debug)]
pub struct StateMessage<StateType>
    where StateType: Clone {

    sequence: usize,
    state: StateType,
}

impl<StateType> StateMessage<StateType>
    where StateType: Clone {

    pub fn new(sequence: usize, state: StateType) -> Self {
        Self{ sequence, state }
    }

    pub fn get_state(self) -> StateType {
        self.state
    }

    pub fn get_sequence(&self) -> usize {
        self.sequence
    }
}

impl<StateType> Clone for StateMessage<StateType>
    where StateType: Clone {

    fn clone(&self) -> Self {
        Self{
            sequence: self.sequence,
            state: self.state.clone(),
        }
    }
}