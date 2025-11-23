use crate::{gametime::FrameIndex, interface::GameTrait};
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerInputMessage<Game: GameTrait> {
    //TODO: rename
    step: FrameIndex,
    server_input: Game::ServerInput,
}

impl<Game: GameTrait> ServerInputMessage<Game> {
    pub fn new(step: FrameIndex, server_input: Game::ServerInput) -> Self {
        Self { step, server_input }
    }

    pub fn get_server_input(self) -> Game::ServerInput {
        self.server_input
    }

    //TODO: rename
    pub fn get_step(&self) -> FrameIndex {
        self.step
    }
}

impl<Game: GameTrait> Clone for ServerInputMessage<Game> {
    fn clone(&self) -> Self {
        Self {
            step: self.step,
            server_input: self.server_input.clone(),
        }
    }
}
