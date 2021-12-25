use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use crate::interface::Game;

#[derive(Serialize, Deserialize, Debug)]
pub struct InputMessage<GameType: Game> {
    sequence: usize,
    player_index: usize,
    input: GameType::InputType
}

impl<GameType: Game> InputMessage<GameType> {

    pub fn new(sequence: usize, player_index: usize, input: GameType::InputType) -> InputMessage<GameType> {
        InputMessage{ sequence, player_index, input }
    }

    pub fn get_step(&self) -> usize {
        self.sequence
    }

    pub fn get_player_index(&self) -> usize {
        self.player_index
    }

    pub fn get_input(self) -> GameType::InputType {
        self.input
    }
}

impl<GameType: Game> Clone for InputMessage<GameType> {

    fn clone(&self) -> Self {
        Self{
            sequence: self.sequence,
            player_index: self.player_index,
            input: self.input.clone()
        }
    }
}

impl<GameType: Game> PartialEq for InputMessage<GameType> {
    fn eq(&self, other: &Self) -> bool {
        self.sequence.eq(&other.sequence) &&
            self.player_index.eq(&other.player_index)
    }
}

impl<GameType: Game> Eq for InputMessage<GameType> {

}

impl<GameType: Game> PartialOrd for InputMessage<GameType> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl<GameType: Game> Ord for InputMessage<GameType> {

    fn cmp(&self, other: &Self) -> Ordering {
        match self.sequence.cmp(&other.sequence) {
            Ordering::Equal => {self.player_index.cmp(&other.player_index)}
            result => {result}
        }
    }
}