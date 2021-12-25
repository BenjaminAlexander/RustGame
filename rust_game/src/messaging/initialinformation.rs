use serde::{Deserialize, Serialize};
use crate::messaging::StateMessage;
use crate::gametime::TimeDuration;
use crate::interface::Game;
use crate::server::ServerConfig;

#[derive(Serialize, Deserialize, Debug)]
pub struct InitialInformation<GameType: Game> {
    server_config: ServerConfig,
    player_count: usize,
    player_index: usize,
    state: GameType::StateType,
}

impl<GameType: Game> InitialInformation<GameType> {

    pub fn new(server_config: ServerConfig,
               player_count: usize,
               player_index: usize,
               state: GameType::StateType) -> Self {

        return Self{
            server_config,
            player_count,
            player_index,
            state
        };
    }

    pub fn get_state(&self) -> &GameType::StateType {
        &self.state
    }

    pub fn move_state(self) -> GameType::StateType {
        self.state
    }

    pub fn get_player_count(&self) -> usize {
        self.player_count
    }

    pub fn get_player_index(&self) -> usize {
        self.player_index
    }

    pub fn get_server_config(&self) -> &ServerConfig {
        return &self.server_config;
    }

    pub fn move_server_config(self) -> ServerConfig {
        return self.server_config;
    }
}

impl<GameType: Game> Clone for InitialInformation<GameType> {

    fn clone(&self) -> Self {
        Self{
            server_config: self.server_config.clone(),
            player_count: self.player_count,
            player_index: self.player_index,
            state: self.state.clone(),
        }
    }
}