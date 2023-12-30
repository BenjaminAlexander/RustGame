use serde::{Deserialize, Serialize};

use crate::interface::GameTrait;
use crate::messaging::InitialInformation;

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "")]
pub enum ToClientMessageTCP<Game: GameTrait> {

    //TODO: see if these can be borrowed
    InitialInformation(InitialInformation<Game>)
}