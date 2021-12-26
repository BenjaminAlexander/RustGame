use serde::{Deserialize, Serialize};
use crate::interface::GameTrait;

#[derive(Serialize, Deserialize, Debug)]
pub struct StateMessage<GameType: GameTrait> {
    sequence: usize,
    state: GameType::StateType,
}

impl<GameType: GameTrait> StateMessage<GameType> {

    pub fn new(sequence: usize, state: GameType::StateType) -> Self {
        Self{ sequence, state }
    }

    pub fn get_state(self) -> GameType::StateType {
        self.state
    }

    pub fn get_sequence(&self) -> usize {
        self.sequence
    }
}

impl<GameType: GameTrait> Clone for StateMessage<GameType> {

    fn clone(&self) -> Self {
        Self{
            sequence: self.sequence,
            state: self.state.clone(),
        }
    }
}