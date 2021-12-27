use serde::{Deserialize, Serialize};
use crate::interface::GameTrait;

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerInputMessage<Game: GameTrait> {
    step: usize,
    server_input: Game::ServerInput,
}

impl<Game: GameTrait> ServerInputMessage<Game> {

    pub fn new(step: usize, server_input: Game::ServerInput) -> Self {
        Self{ step, server_input }
    }

    pub fn get_server_input(self) -> Game::ServerInput {
        self.server_input
    }

    pub fn get_step(&self) -> usize {
        self.step
    }
}

impl<Game: GameTrait> Clone for ServerInputMessage<Game> {

    fn clone(&self) -> Self {
        Self{
            step: self.step,
            server_input: self.server_input.clone(),
        }
    }
}