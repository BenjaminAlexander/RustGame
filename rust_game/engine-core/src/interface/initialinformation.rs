use crate::interface::GameTrait;
use crate::server::ServerConfig;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct InitialInformation<Game: GameTrait> {
    server_config: ServerConfig,
    player_count: usize,
    player_index: usize,
    state: Game::State,
}

impl<Game: GameTrait> InitialInformation<Game> {
    pub fn new(
        server_config: ServerConfig,
        player_count: usize,
        player_index: usize,
        state: Game::State,
    ) -> Self {
        return Self {
            server_config,
            player_count,
            player_index,
            state,
        };
    }

    pub fn get_state(&self) -> &Game::State {
        &self.state
    }

    pub fn move_state(self) -> Game::State {
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

impl<Game: GameTrait> Clone for InitialInformation<Game> {
    fn clone(&self) -> Self {
        Self {
            server_config: self.server_config.clone(),
            player_count: self.player_count,
            player_index: self.player_index,
            state: self.state.clone(),
        }
    }
}
