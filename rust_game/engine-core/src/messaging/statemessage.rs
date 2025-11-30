use std::cmp::Ordering;

use crate::{
    game_time::FrameIndex,
    interface::GameTrait,
};
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StateMessage<Game: GameTrait> {
    frame_index: FrameIndex,
    state: Game::State,
}

impl<Game: GameTrait> StateMessage<Game> {
    pub fn new(frame_index: FrameIndex, state: Game::State) -> Self {
        Self {
            frame_index,
            state,
        }
    }

    pub fn get_state(&self) -> &Game::State {
        &self.state
    }

    pub fn take_state(self) -> Game::State {
        self.state
    }

    pub fn get_frame_index(&self) -> FrameIndex {
        self.frame_index
    }
}

impl<Game: GameTrait> PartialEq for StateMessage<Game> {
    fn eq(&self, other: &Self) -> bool {
        return self.frame_index.eq(&other.frame_index);
    }
}

impl<Game: GameTrait> Eq for StateMessage<Game> {}

impl<Game: GameTrait> PartialOrd for StateMessage<Game> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return Some(self.cmp(&other));
    }
}

impl<Game: GameTrait> Ord for StateMessage<Game> {
    fn cmp(&self, other: &Self) -> Ordering {
        return self.frame_index.cmp(&other.frame_index);
    }
}
