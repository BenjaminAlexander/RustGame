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
    is_authoritative: bool
}

impl<Game: GameTrait> ToServerInputMessage<Game> {
    pub fn new(frame_index: FrameIndex, player_index: usize, input: Game::ClientInput, is_authoritative: bool) -> Self {
        Self {
            frame_index,
            player_index,
            input,
            is_authoritative
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

    pub fn is_authoritative(&self) -> bool {
        self.is_authoritative
    }

    pub fn to_client_message(self) -> ToClientInputMessage<Game> {
        let input = match self.is_authoritative {
            true => ToClientInput::Authoritative(self.input),
            false => ToClientInput::NonAuthoritative(self.input),
        };
        
        ToClientInputMessage::new(self.frame_index, self.player_index, input)
    }
}

//TODO: document this
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ToClientInput<T> {
    AuthoritativeMissing,
    Authoritative(T),
    NonAuthoritative(T)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToClientInputMessage<Game: GameTrait> {
    frame_index: FrameIndex,
    player_index: usize,
    input: ToClientInput<Game::ClientInput>,
}

impl<Game: GameTrait> ToClientInputMessage<Game> {
    pub fn new(
        frame_index: FrameIndex,
        player_index: usize,
        input: ToClientInput<Game::ClientInput>,
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

    pub fn get_input(&self) -> &ToClientInput<Game::ClientInput> {
        &self.input
    }

    pub fn take_input(self) -> ToClientInput<Game::ClientInput> {
        self.input
    }
}
