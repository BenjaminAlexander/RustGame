use serde::{Deserialize, Serialize};
use crate::interface::GameTrait;

#[derive(Serialize, Deserialize, Debug)]
pub struct StateMessage<Game: GameTrait> {
    sequence: usize,
    state: Game::StateType,
}

impl<Game: GameTrait> StateMessage<Game> {

    pub fn new(sequence: usize, state: Game::StateType) -> Self {
        Self{ sequence, state }
    }

    pub fn get_state(self) -> Game::StateType {
        self.state
    }

    pub fn get_sequence(&self) -> usize {
        self.sequence
    }
}

impl<Game: GameTrait> Clone for StateMessage<Game> {

    fn clone(&self) -> Self {
        Self{
            sequence: self.sequence,
            state: self.state.clone(),
        }
    }
}