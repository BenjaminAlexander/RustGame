use serde::{Deserialize, Serialize};

use crate::gametime::TimeMessage;
use crate::interface::Game;
use crate::messaging::{InputMessage, StateMessage, InitialInformation};

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "")]
pub enum ToClientMessageTCP<GameType: Game> {

    //TODO: see if these can be borrowed
    InitialInformation(InitialInformation<GameType>)
}