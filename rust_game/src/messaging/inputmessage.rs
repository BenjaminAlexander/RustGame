use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Serialize, Deserialize, Debug)]
pub struct InputMessage<InputType>
    where InputType: Clone {

    sequence: usize,
    player_index: usize,
    input: InputType
}

impl<InputType> InputMessage<InputType>
    where InputType: Clone {

    pub fn new(sequence: usize, player_index: usize, input: InputType) -> InputMessage<InputType> {
        InputMessage{ sequence, player_index, input }
    }

    pub fn get_step(&self) -> usize {
        self.sequence
    }

    pub fn get_player_index(&self) -> usize {
        self.player_index
    }

    pub fn get_input(self) -> InputType {
        self.input
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

impl<InputType> PartialEq for InputMessage<InputType>
    where InputType: Clone {
    fn eq(&self, other: &Self) -> bool {
        self.sequence.eq(&other.sequence) &&
            self.player_index.eq(&other.player_index)
    }
}

impl<InputType> Eq for InputMessage<InputType>
    where InputType: Clone {

}

impl<InputType> PartialOrd for InputMessage<InputType>
    where InputType: Clone {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl<InputType> Ord for InputMessage<InputType>
    where InputType: Clone {

    fn cmp(&self, other: &Self) -> Ordering {
        match self.sequence.cmp(&other.sequence) {
            Ordering::Equal => {self.player_index.cmp(&other.player_index)}
            result => {result}
        }
    }
}