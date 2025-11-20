use serde::{
    Deserialize,
    Serialize,
};

use crate::gametime::FrameDuration;

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct ServerConfig {
    game_timer_config: FrameDuration,
}

impl ServerConfig {
    pub fn new(game_timer_config: FrameDuration) -> Self {
        return Self { game_timer_config };
    }

    pub fn get_game_timer_config(&self) -> &FrameDuration {
        return &self.game_timer_config;
    }
}
