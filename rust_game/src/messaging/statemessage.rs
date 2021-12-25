use serde::{Deserialize, Serialize};
use crate::interface::Game;

#[derive(Serialize, Deserialize, Debug)]
pub struct StateMessage<GameType: Game> {
    sequence: usize,
    state: GameType::StateType,
}

impl<GameType: Game> StateMessage<GameType> {

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

impl<GameType: Game> Clone for StateMessage<GameType> {

    fn clone(&self) -> Self {
        Self{
            sequence: self.sequence,
            state: self.state.clone(),
        }
    }
}