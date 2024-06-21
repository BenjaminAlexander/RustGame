use serde::{Deserialize, Serialize};

use crate::gametime::TimeMessage;
use crate::interface::GameTrait;
use crate::messaging::{InputMessage, ServerInputMessage, StateMessage};

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "")]
pub enum ToClientMessageUDP<Game: GameTrait> {
    //TODO: see if these can be borrowed
    TimeMessage(TimeMessage),
    InputMessage(InputMessage<Game>),
    ServerInputMessage(ServerInputMessage<Game>),
    StateMessage(StateMessage<Game>),
}
