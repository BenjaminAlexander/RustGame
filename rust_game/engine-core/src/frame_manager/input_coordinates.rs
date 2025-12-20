use serde::{Deserialize, Serialize};

use crate::FrameIndex;

/// The coordinates of an input in the [FrameManager][super::FrameManager]
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct InputCoordinates {
    frame_index: FrameIndex,
    player_index: usize,
}

impl InputCoordinates {
    pub fn new(frame_index: FrameIndex, player_index: usize) -> Self {
        Self {
            frame_index,
            player_index,
        }
    }

    pub fn get_frame_index(&self) -> FrameIndex {
        self.frame_index
    }

    pub fn get_player_index(&self) -> usize {
        self.player_index
    }

    pub fn previous(&self) -> InputCoordinates {
        Self {
            frame_index: self.frame_index.previous(),
            player_index: self.player_index
        }
    }
}