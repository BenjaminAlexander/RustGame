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
pub enum UdpToServerMessage<Game: GameTrait> {
    //TODO: see if these can be borrowed
    PingRequest(PingRequest),
    Input(InputMessage<Game>),
}

impl<Game: GameTrait> UdpToServerMessage<Game> {
    pub fn get_player_index(&self) -> usize {
        return match self {
            UdpToServerMessage::Input(input_message) => {
                input_message.get_player_index()
            }
            UdpToServerMessage::PingRequest(ping_request) => ping_request.get_player_index(),
        };
    }
}
