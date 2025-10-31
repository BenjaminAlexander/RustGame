use serde::{
    Deserialize,
    Serialize,
};

use crate::gametime::GameTimerConfig;

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct ServerConfig {
    game_timer_config: GameTimerConfig,
}

impl ServerConfig {
    pub fn new(game_timer_config: GameTimerConfig) -> Self {
        return Self { game_timer_config };
    }

    pub fn get_game_timer_config(&self) -> &GameTimerConfig {
        return &self.game_timer_config;
    }
}
