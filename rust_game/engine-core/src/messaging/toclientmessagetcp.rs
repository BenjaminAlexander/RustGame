use crate::interface::GameTrait;
use crate::interface::InitialInformation;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "")]
pub enum ToClientMessageTCP<Game: GameTrait> {
    //TODO: see if these can be borrowed
    InitialInformation(InitialInformation<Game>),
}
