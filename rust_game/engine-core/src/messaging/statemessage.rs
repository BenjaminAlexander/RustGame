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
    frame_index: FrameIndex,
    state: Game::State,
}

impl<Game: GameTrait> StateMessage<Game> {
    pub fn new(sequence: FrameIndex, state: Game::State) -> Self {
        Self {
            frame_index: sequence,
            state,
        }
    }

    pub fn get_state(self) -> Game::State {
        self.state
    }

    pub fn get_frame_index(&self) -> FrameIndex {
        self.frame_index
    }
}

impl<Game: GameTrait> Clone for StateMessage<Game> {
    fn clone(&self) -> Self {
        Self {
            frame_index: self.frame_index,
            state: self.state.clone(),
        }
    }
}
