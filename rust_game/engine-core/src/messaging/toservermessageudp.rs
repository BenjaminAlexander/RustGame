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
pub enum UdpToServerMessage {
    PingRequest(PingRequest),

    //TODO: maybe add variant for an unfragmented message?
    //TODO: can this vec be borrowed?
    Fragment(Vec<u8>),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "")]
pub enum FragmentableUdpToServerMessage<Game: GameTrait> {
    //TODO: see if these can be borrowed
    //TODO: remove hello
    Hello { player_index: usize },
    Input(InputMessage<Game>),
    /*
    PingRequest{
        player_index: usize,
        ping_request: PingRequest
    }
    */
}

impl<Game: GameTrait> FragmentableUdpToServerMessage<Game> {
    pub fn get_player_index(&self) -> usize {
        return match self {
            FragmentableUdpToServerMessage::Hello { player_index } => *player_index,
            FragmentableUdpToServerMessage::Input(input_message) => {
                input_message.get_player_index()
            }
            //ToServerMessageUDP::PingRequest { player_index, .. } => *player_index,
        };
    }
}
