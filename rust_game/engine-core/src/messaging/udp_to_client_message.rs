use serde::{
    Deserialize,
    Serialize,
};

use crate::game_time::PingResponse;
use crate::interface::GameTrait;
use crate::messaging::{
    InputMessage,
    StateMessage,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "")]
pub enum UdpToClientMessage<Game: GameTrait> {
    //TODO: see if these can be borrowed
    PingResponse(PingResponse),
    InputMessage(InputMessage<Game>),
    StateMessage(StateMessage<Game>),
}
