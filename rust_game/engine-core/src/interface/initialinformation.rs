use {
    crate::{
        Play,
        game_time::FrameDuration,
        server::ServerConfig,
    },
    serde::{
        Deserialize,
        Serialize,
    },
};

/// A collection of information that is known before the game starts
#[derive(Serialize, Deserialize, Debug)]
pub struct InitialInformation<Game: Play> {
    server_config: ServerConfig,
    player_count: usize,
    player_index: usize,
    initial_state: Game::State,
}

impl<Game: Play> InitialInformation<Game> {
    pub fn new(
        server_config: ServerConfig,
        player_count: usize,
        player_index: usize,
        initial_state: Game::State,
    ) -> Self {
        return Self {
            server_config,
            player_count,
            player_index,
            initial_state,
        };
    }

    /// Returns the initial [State][Play::State] of the game
    pub fn get_initial_state(&self) -> &Game::State {
        &self.initial_state
    }

    /// Returns the number of players in this game
    pub fn get_player_count(&self) -> usize {
        self.player_count
    }

    /// Returns the index of the local player
    pub fn get_player_index(&self) -> usize {
        self.player_index
    }

    /// Returns the [ServerConfig] of the game
    pub(crate) fn get_server_config(&self) -> &ServerConfig {
        return &self.server_config;
    }

    /// Returns the [FrameDuration] of the game
    pub fn get_frame_duration(&self) -> &FrameDuration {
        return &self.server_config.get_frame_duration();
    }
}

impl<Game: Play> Clone for InitialInformation<Game> {
    fn clone(&self) -> Self {
        Self {
            server_config: self.server_config.clone(),
            player_count: self.player_count,
            player_index: self.player_index,
            initial_state: self.initial_state.clone(),
        }
    }
}
