use crate::{
    game_time::FrameIndex,
    interface::GameTrait,
};
use serde::{
    Deserialize,
    Serialize,
};
use std::cmp::Ordering;

#[derive(Serialize, Deserialize, Debug)]
pub struct InputMessage<Game: GameTrait> {
    //TODO: rename
    sequence: FrameIndex,
    player_index: usize,
    input: Game::ClientInput,
}

impl<Game: GameTrait> InputMessage<Game> {
    pub fn new(
        sequence: FrameIndex,
        player_index: usize,
        input: Game::ClientInput,
    ) -> InputMessage<Game> {
        InputMessage {
            sequence,
            player_index,
            input,
        }
    }

    //TODO: rename
    pub fn get_step(&self) -> FrameIndex {
        self.sequence
    }

    pub fn get_player_index(&self) -> usize {
        self.player_index
    }

    pub fn get_input(self) -> Game::ClientInput {
        self.input
    }
}

impl<Game: GameTrait> Clone for InputMessage<Game> {
    fn clone(&self) -> Self {
        Self {
            sequence: self.sequence,
            player_index: self.player_index,
            input: self.input.clone(),
        }
    }
}

impl<Game: GameTrait> PartialEq for InputMessage<Game> {
    fn eq(&self, other: &Self) -> bool {
        self.sequence.eq(&other.sequence) && self.player_index.eq(&other.player_index)
    }
}

impl<Game: GameTrait> Eq for InputMessage<Game> {}

impl<Game: GameTrait> PartialOrd for InputMessage<Game> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl<Game: GameTrait> Ord for InputMessage<Game> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.sequence.cmp(&other.sequence) {
            Ordering::Equal => self.player_index.cmp(&other.player_index),
            result => result,
        }
    }
}
