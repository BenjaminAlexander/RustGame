use crate::{
    game_time::FrameIndex,
    interface::GameTrait,
};
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct StateMessage<Game: GameTrait> {
    //TODO: rename
    sequence: FrameIndex,
    state: Game::State,
}

impl<Game: GameTrait> StateMessage<Game> {
    pub fn new(sequence: FrameIndex, state: Game::State) -> Self {
        Self { sequence, state }
    }

    pub fn get_state(self) -> Game::State {
        self.state
    }

    //TODO: rename
    pub fn get_sequence(&self) -> FrameIndex {
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
