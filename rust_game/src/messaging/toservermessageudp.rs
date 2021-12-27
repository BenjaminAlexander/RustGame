use serde::{Deserialize, Serialize};
use crate::interface::GameTrait;

use super::InputMessage;

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "")]
pub enum ToServerMessageUDP<Game: GameTrait> {

    //TODO: see if these can be borrowed
    //TODO: remove hello
    Hello{player_index: usize},
    Input(InputMessage<Game>)
}

impl<Game: GameTrait> ToServerMessageUDP<Game> {

    pub fn get_player_index(&self) -> usize {
        return match self {
            ToServerMessageUDP::Hello { player_index } => player_index.clone(),
            ToServerMessageUDP::Input(input_message) => input_message.get_player_index()
        }
    }
}