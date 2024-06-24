use crate::interface::GameTrait;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct StateMessage<Game: GameTrait> {
    sequence: usize,
    state: Game::State,
}

impl<Game: GameTrait> StateMessage<Game> {
    pub fn new(sequence: usize, state: Game::State) -> Self {
        Self { sequence, state }
    }

    pub fn get_state(self) -> Game::State {
        self.state
    }

    pub fn get_sequence(&self) -> usize {
        self.sequence
    }
}

impl<Game: GameTrait> Clone for StateMessage<Game> {
    fn clone(&self) -> Self {
        Self {
            sequence: self.sequence,
            state: self.state.clone(),
        }
    }
}
