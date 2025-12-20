use crate::{
    frame_manager::{ClientProvidedInput, InputCoordinates, PresentInput}, interface::GameTrait
};
use serde::{
    Deserialize,
    Serialize,
};

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

    pub fn take(self) -> (InputCoordinates, ClientProvidedInput<Game::ClientInput>) {
        (self.input_coordinates, self.input)
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

    pub fn take(self) -> (InputCoordinates, PresentInput<Game::ClientInput>) {
        (self.input_coordinates, self.input)
    }
}
