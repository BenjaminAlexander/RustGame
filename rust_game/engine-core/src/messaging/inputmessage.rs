use crate::{
    Play, frame_manager::{
        ClientProvidedInput,
        InputCoordinates,
        PresentInput,
    }
};
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(bound = "")]
pub struct ToServerInputMessage<Game: Play> {
    input_coordinates: InputCoordinates,
    input: ClientProvidedInput<Game::InputAggregate>,
}

impl<Game: Play> ToServerInputMessage<Game> {
    pub fn new(input_coordinates: InputCoordinates, input: ClientProvidedInput<Game::InputAggregate>) -> Self {
        Self {
            input_coordinates,
            input,
        }
    }

    pub fn get_input_coordinates(&self) -> &InputCoordinates {
        &self.input_coordinates
    }

    pub fn get_input(&self) -> &ClientProvidedInput<Game::InputAggregate> {
        &self.input
    }

    pub fn take(self) -> (InputCoordinates, ClientProvidedInput<Game::InputAggregate>) {
        (self.input_coordinates, self.input)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(bound = "")]
pub struct ToClientInputMessage<Game: Play> {
    input_coordinates: InputCoordinates,
    input: PresentInput<Game::InputAggregate>,
}

impl<Game: Play> ToClientInputMessage<Game> {
    pub fn new(input_coordinates: InputCoordinates, input: PresentInput<Game::InputAggregate>) -> Self {
        Self {
            input_coordinates,
            input,
        }
    }

    pub fn take(self) -> (InputCoordinates, PresentInput<Game::InputAggregate>) {
        (self.input_coordinates, self.input)
    }
}
