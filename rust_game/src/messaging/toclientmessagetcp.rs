use serde::{Deserialize, Serialize};

use crate::gametime::TimeMessage;
use crate::messaging::{InputMessage, StateMessage, InitialInformation};

#[derive(Serialize, Deserialize, Debug)]
pub enum ToClientMessageTCP<StateType, InputType>
    where InputType: Clone,
          StateType: Clone {

    //TODO: see if these can be borrowed
    //TODO: remove messages that have been moved to UDP
    InitialInformation(InitialInformation<StateType>),
    TimeMessage(TimeMessage),
    InputMessage(InputMessage<InputType>),
    StateMessage(StateMessage<StateType>)
}