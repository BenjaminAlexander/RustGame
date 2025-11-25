use serde::{
    Deserialize,
    Serialize,
};

use crate::interface::GameTrait;
use crate::messaging::{
    InputMessage,
    ServerInputMessage,
    StateMessage,
};
use crate::FrameIndex;

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
