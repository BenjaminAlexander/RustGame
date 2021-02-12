use serde::{Deserialize, Serialize};

use crate::gametime::TimeMessage;
use crate::messaging::{InputMessage, StateMessage, InitialInformation};

#[derive(Serialize, Deserialize, Debug)]
pub enum ToClientMessageTCP<StateType>
    where StateType: Clone {

    //TODO: see if these can be borrowed
    InitialInformation(InitialInformation<StateType>)
}