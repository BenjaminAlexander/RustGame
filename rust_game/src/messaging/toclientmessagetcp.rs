use serde::{Deserialize, Serialize};

use crate::interface::GameTrait;
use crate::messaging::InitialInformation;

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "")]
pub enum ToClientMessageTCP<GameType: GameTrait> {

    //TODO: see if these can be borrowed
    InitialInformation(InitialInformation<GameType>)
}