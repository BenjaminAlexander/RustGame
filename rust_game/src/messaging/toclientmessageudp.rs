use serde::{Deserialize, Serialize};

use crate::gametime::TimeMessage;
use crate::interface::GameTrait;
use crate::messaging::{InputMessage, StateMessage, ServerInputMessage};

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "")]
pub enum ToClientMessageUDP<GameType: GameTrait> {

    //TODO: see if these can be borrowed
    TimeMessage(TimeMessage),
    InputMessage(InputMessage<GameType>),
    ServerInputMessage(ServerInputMessage<GameType>),
    StateMessage(StateMessage<GameType>)
}