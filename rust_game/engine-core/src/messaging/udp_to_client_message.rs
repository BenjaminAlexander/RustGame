use serde::{
    Deserialize,
    Serialize,
};

use crate::Play;
use crate::frame_manager::FrameIndexAndState;
use crate::game_time::PingResponse;
use crate::messaging::ToClientInputMessage;

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "")]
pub enum UdpToClientMessage<Game: Play> {
    //TODO: see if these can be borrowed
    PingResponse(PingResponse),
    InputMessage(ToClientInputMessage<Game>),
    StateMessage(FrameIndexAndState<Game>),
}
