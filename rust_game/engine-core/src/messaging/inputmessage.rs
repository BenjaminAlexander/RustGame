use crate::{
    frame_manager::{ClientProvidedInput, PresentInput}, game_time::FrameIndex, interface::GameTrait
};
use serde::{
    Deserialize,
    Serialize,
};

//TODO: move to frame manager
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct InputCoordinates {
    frame_index: FrameIndex,
    player_index: usize,
}

impl InputCoordinates {
    pub fn new(frame_index: FrameIndex, player_index: usize) -> Self {
        Self {
            frame_index,
            player_index,
        }
    }

    pub fn get_frame_index(&self) -> FrameIndex {
        self.frame_index
    }

    pub fn get_player_index(&self) -> usize {
        self.player_index
    }

    pub fn previous(&self) -> InputCoordinates {
        Self {
            frame_index: self.frame_index.previous(),
            player_index: self.player_index
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToServerInputMessage<Game: GameTrait> {
    input_coordinates: InputCoordinates,
    input: ClientProvidedInput<Game::ClientInput>,
}

impl<Game: GameTrait> ToServerInputMessage<Game> {
    pub fn new(input_coordinates: InputCoordinates, input: ClientProvidedInput<Game::ClientInput>) -> Self {
        Self {
            input_coordinates,
            input
        }
    }

    pub fn get_input_coordinates(&self) -> &InputCoordinates {
        &self.input_coordinates
    }

    pub fn get_input(&self) -> &ClientProvidedInput<Game::ClientInput> {
        &self.input
    }

    pub fn take_input(self) -> ClientProvidedInput<Game::ClientInput> {
        self.input
    }

    pub fn take(self) -> (InputCoordinates, ClientProvidedInput<Game::ClientInput>) {
        (self.input_coordinates, self.input)
    }

    pub fn to_client_message(self) -> ToClientInputMessage<Game> {
        ToClientInputMessage::new(
            self.input_coordinates, 
            PresentInput::ClientProvided(self.input)
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToClientInputMessage<Game: GameTrait> {
    input_coordinates: InputCoordinates,
    input: PresentInput<Game::ClientInput>,
}

impl<Game: GameTrait> ToClientInputMessage<Game> {
    pub fn new(
        input_coordinates: InputCoordinates,
        input: PresentInput<Game::ClientInput>,
    ) -> Self {
        Self {
            input_coordinates,
            input,
        }
    }

    pub fn get_input_coordinates(&self) -> &InputCoordinates {
        &self.input_coordinates
    }

    pub fn get_input(&self) -> &PresentInput<Game::ClientInput> {
        &self.input
    }

    pub fn take_input(self) -> PresentInput<Game::ClientInput> {
        self.input
    }

    pub fn take(self) -> (InputCoordinates, PresentInput<Game::ClientInput>) {
        (self.input_coordinates, self.input)
    }
}
