use crate::{
    Play,
    interface::InitialInformation,
};
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(bound = "")]
pub enum ToClientMessageTCP<Game: Play> {
    //TODO: see if these can be borrowed
    InitialInformation(InitialInformation<Game>),
}
