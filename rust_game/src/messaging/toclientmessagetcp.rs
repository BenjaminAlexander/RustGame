use serde::{Deserialize, Serialize};

use crate::interface::Game;
use crate::messaging::InitialInformation;

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "")]
pub enum ToClientMessageTCP<GameType: Game> {

    //TODO: see if these can be borrowed
    InitialInformation(InitialInformation<GameType>)
}