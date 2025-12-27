use std::cmp::Ordering;

use crate::{
    Play,
    game_time::FrameIndex,
};
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FrameIndexAndState<Game: Play> {
    frame_index: FrameIndex,
    state: Game::State,
}

impl<Game: Play> FrameIndexAndState<Game> {
    pub fn new(frame_index: FrameIndex, state: Game::State) -> Self {
        Self { frame_index, state }
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

impl<Game: Play> PartialEq for FrameIndexAndState<Game> {
    fn eq(&self, other: &Self) -> bool {
        return self.frame_index.eq(&other.frame_index);
    }
}

impl<Game: Play> Eq for FrameIndexAndState<Game> {}

impl<Game: Play> PartialOrd for FrameIndexAndState<Game> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return Some(self.cmp(&other));
    }
}

impl<Game: Play> Ord for FrameIndexAndState<Game> {
    fn cmp(&self, other: &Self) -> Ordering {
        return self.frame_index.cmp(&other.frame_index);
    }
}
