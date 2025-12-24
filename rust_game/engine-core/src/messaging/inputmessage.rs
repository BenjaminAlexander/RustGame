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
    input: ClientProvidedInput<Game>,
}

impl<Game: Play> ToServerInputMessage<Game> {
    pub fn new(input_coordinates: InputCoordinates, input: ClientProvidedInput<Game>) -> Self {
        Self {
            input_coordinates,
            input,
        }
    }

    pub fn get_input_coordinates(&self) -> &InputCoordinates {
        &self.input_coordinates
    }

    pub fn get_input(&self) -> &ClientProvidedInput<Game> {
        &self.input
    }

    pub fn take(self) -> (InputCoordinates, ClientProvidedInput<Game>) {
        (self.input_coordinates, self.input)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(bound = "")]
pub struct ToClientInputMessage<Game: Play> {
    input_coordinates: InputCoordinates,
    input: PresentInput<Game>,
}

impl<Game: Play> ToClientInputMessage<Game> {
    pub fn new(input_coordinates: InputCoordinates, input: PresentInput<Game>) -> Self {
        Self {
            input_coordinates,
            input,
        }
    }

    pub fn take(self) -> (InputCoordinates, PresentInput<Game>) {
        (self.input_coordinates, self.input)
    }
}
