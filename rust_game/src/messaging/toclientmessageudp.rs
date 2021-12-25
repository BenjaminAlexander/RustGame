use serde::{Deserialize, Serialize};

use crate::gametime::TimeMessage;
use crate::interface::Game;
use crate::messaging::{InputMessage, StateMessage, InitialInformation, ServerInputMessage};

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "")]
pub enum ToClientMessageUDP<GameType: Game> {

    //TODO: see if these can be borrowed
    TimeMessage(TimeMessage),
    InputMessage(InputMessage<GameType>),
    ServerInputMessage(ServerInputMessage<GameType>),
    StateMessage(StateMessage<GameType>)
}