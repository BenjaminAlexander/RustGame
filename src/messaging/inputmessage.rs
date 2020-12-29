use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct InputMessage<InputType>
    where InputType: Clone {

    sequence: i32,
    player_index: i8,
    input: InputType
}

impl<InputType> InputMessage<InputType>
    where InputType: Clone {

    pub fn new(sequence: i32, player_index: i8, input: InputType) -> InputMessage<InputType> {
        InputMessage{ sequence, player_index, input }
    }
}

impl<InputType> Clone for InputMessage<InputType>
    where InputType: Clone {

    fn clone(&self) -> Self {
        Self{
            sequence: self.sequence,
            player_index: self.player_index,
            input: self.input.clone()
        }
    }
}