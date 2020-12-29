use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct InputMessage<InputType>
    where InputType: Clone {

    sequence: i32,
    playerIndex: i8,
    input: InputType
}

impl<InputType> InputMessage<InputType>
    where InputType: Clone {

    pub fn new(sequence: i32, playerIndex: i8, input: InputType) -> InputMessage<InputType> {
        InputMessage{ sequence, playerIndex, input }
    }
}

impl<InputType> Clone for InputMessage<InputType>
    where InputType: Clone {

    fn clone(&self) -> Self {
        Self{
            sequence: self.sequence,
            playerIndex: self.playerIndex,
            input: self.input.clone()
        }
    }
}