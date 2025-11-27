use crate::{
    game_time::PingRequest,
    interface::GameTrait,
};
use serde::{
    Deserialize,
    Serialize,
};

use super::InputMessage;

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "")]
//TODO: rename
pub enum FragmentableUdpToServerMessage<Game: GameTrait> {
    //TODO: see if these can be borrowed
    PingRequest(PingRequest),
    Input(InputMessage<Game>),
}

impl<Game: GameTrait> FragmentableUdpToServerMessage<Game> {
    pub fn get_player_index(&self) -> usize {
        return match self {
            FragmentableUdpToServerMessage::Input(input_message) => {
                input_message.get_player_index()
            }
            FragmentableUdpToServerMessage::PingRequest(ping_request) => ping_request.get_player_index(),
        };
    }
}
