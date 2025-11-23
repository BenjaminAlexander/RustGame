use serde::{
    Deserialize,
    Serialize,
};

use crate::FrameIndex;
use crate::interface::GameTrait;
use crate::messaging::{
    InputMessage,
    ServerInputMessage,
    StateMessage,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "")]
pub enum ToClientMessageUDP<Game: GameTrait> {
    //TODO: see if these can be borrowed
    //TODO: rename
    TimeMessage(FrameIndex),
    InputMessage(InputMessage<Game>),
    ServerInputMessage(ServerInputMessage<Game>),
    StateMessage(StateMessage<Game>),
}
