use serde::{Deserialize, Serialize};
use crate::interface::GameTrait;

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerInputMessage<GameType: GameTrait> {
    step: usize,
    server_input: GameType::ServerInputType,
}

impl<GameType: GameTrait> ServerInputMessage<GameType> {

    pub fn new(step: usize, server_input: GameType::ServerInputType) -> Self {
        Self{ step, server_input }
    }

    pub fn get_server_input(self) -> GameType::ServerInputType {
        self.server_input
    }

    pub fn get_step(&self) -> usize {
        self.step
    }
}

impl<GameType: GameTrait> Clone for ServerInputMessage<GameType> {

    fn clone(&self) -> Self {
        Self{
            step: self.step,
            server_input: self.server_input.clone(),
        }
    }
}