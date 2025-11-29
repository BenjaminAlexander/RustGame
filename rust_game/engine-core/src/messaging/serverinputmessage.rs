use crate::{
    game_time::FrameIndex,
    interface::GameTrait,
};
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerInputMessage<Game: GameTrait> {
    frame_index: FrameIndex,
    server_input: Game::ServerInput,
}

impl<Game: GameTrait> ServerInputMessage<Game> {
    pub fn new(step: FrameIndex, server_input: Game::ServerInput) -> Self {
        Self {
            frame_index: step,
            server_input,
        }
    }

    pub fn get_server_input(self) -> Game::ServerInput {
        self.server_input
    }

    pub fn get_frame_index(&self) -> FrameIndex {
        self.frame_index
    }
}

impl<Game: GameTrait> Clone for ServerInputMessage<Game> {
    fn clone(&self) -> Self {
        Self {
            frame_index: self.frame_index,
            server_input: self.server_input.clone(),
        }
    }
}
