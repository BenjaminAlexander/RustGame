use serde::{
    Deserialize,
    Serialize,
};

use crate::game_time::PingResponse;
use crate::interface::GameTrait;
use crate::messaging::{
    InputMessage,
    ServerInputMessage,
    StateMessage,
};
use crate::FrameIndex;

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "")]
pub enum UdpToClientMessage {
    PingResponse(PingResponse),

    //TODO: maybe add variant for an unfragmented message?
    //TODO: can this vec be borrowed?
    Fragment(Vec<u8>)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "")]
pub enum ToClientMessageUDP<Game: GameTrait> {
    //TODO: see if these can be borrowed
    
    //TODO: remove TimeMessage
    TimeMessage(FrameIndex),
    InputMessage(InputMessage<Game>),
    ServerInputMessage(ServerInputMessage<Game>),
    StateMessage(StateMessage<Game>),
}
