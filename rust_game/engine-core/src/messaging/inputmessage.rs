use crate::{
    game_time::FrameIndex,
    interface::GameTrait,
};
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToServerInputMessage<Game: GameTrait> {
    frame_index: FrameIndex,
    player_index: usize,
    input: Game::ClientInput,
}

impl<Game: GameTrait> ToServerInputMessage<Game> {
    pub fn new(frame_index: FrameIndex, player_index: usize, input: Game::ClientInput) -> Self {
        Self {
            frame_index,
            player_index,
            input,
        }
    }

    pub fn get_frame_index(&self) -> FrameIndex {
        self.frame_index
    }

    pub fn get_player_index(&self) -> usize {
        self.player_index
    }

    pub fn get_input(&self) -> &Game::ClientInput {
        &self.input
    }

    pub fn take_input(self) -> Game::ClientInput {
        self.input
    }

    pub fn to_client_message(self) -> ToClientInputMessage<Game> {
        ToClientInputMessage::new(self.frame_index, self.player_index, Some(self.input))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToClientInputMessage<Game: GameTrait> {
    frame_index: FrameIndex,
    player_index: usize,

    /// When this is None, the message means that the server has declared the input
    /// authoritatively missing
    input: Option<Game::ClientInput>,
}

impl<Game: GameTrait> ToClientInputMessage<Game> {
    pub fn new(
        frame_index: FrameIndex,
        player_index: usize,
        input: Option<Game::ClientInput>,
    ) -> Self {
        Self {
            frame_index,
            player_index,
            input,
        }
    }

    pub fn get_frame_index(&self) -> FrameIndex {
        self.frame_index
    }

    pub fn get_player_index(&self) -> usize {
        self.player_index
    }

    pub fn get_input(&self) -> &Option<Game::ClientInput> {
        &self.input
    }

    pub fn take_input(self) -> Option<Game::ClientInput> {
        self.input
    }
}
