use serde::{Deserialize, Serialize};
use crate::messaging::StateMessage;
use crate::gametime::TimeDuration;
use crate::server::ServerConfig;

#[derive(Serialize, Deserialize, Debug)]
pub struct InitialInformation<StateType>
    where StateType: Clone {

    server_config: ServerConfig,
    player_count: usize,
    player_index: usize,
    state: StateType,

}

impl<StateType> InitialInformation<StateType>
    where StateType: Clone {

    pub fn new(server_config: ServerConfig,
               player_count: usize,
               player_index: usize,
               state: StateType) -> Self {

        return Self{
            server_config,
            player_count,
            player_index,
            state
        };
    }

    pub fn get_state(&self) -> &StateType {
        &self.state
    }

    pub fn move_state(self) -> StateType {
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

impl<StateType> Clone for InitialInformation<StateType>
    where StateType: Clone {

    fn clone(&self) -> Self {
        Self{
            server_config: self.server_config.clone(),
            player_count: self.player_count,
            player_index: self.player_index,
            state: self.state.clone(),
        }
    }
}