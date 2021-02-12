use serde::{Deserialize, Serialize};

use super::InputMessage;

#[derive(Serialize, Deserialize, Debug)]
pub enum ToServerMessageUDP<InputType>
    where InputType: Clone {

    //TODO: see if these can be borrowed
    //TODO: remove hello
    Hello{player_index: usize},
    Input(InputMessage<InputType>)
}

impl<InputType> ToServerMessageUDP<InputType>
    where InputType: Clone {

    pub fn get_player_index(&self) -> usize {
        return match self {
            ToServerMessageUDP::Hello { player_index } => player_index.clone(),
            ToServerMessageUDP::Input(input_message) => input_message.get_player_index()
        }
    }
}