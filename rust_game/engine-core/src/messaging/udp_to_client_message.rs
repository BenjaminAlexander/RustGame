use serde::{
    Deserialize,
    Serialize,
};

use crate::game_time::PingResponse;
use crate::interface::GameTrait;
use crate::messaging::{
    FrameIndexAndState,
    ToClientInputMessage,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "")]
pub enum UdpToClientMessage<Game: GameTrait> {
    //TODO: see if these can be borrowed
    PingResponse(PingResponse),
    InputMessage(ToClientInputMessage<Game>),
    StateMessage(FrameIndexAndState<Game>),
}
