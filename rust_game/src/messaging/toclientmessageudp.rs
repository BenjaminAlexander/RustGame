use serde::{Deserialize, Serialize};

use crate::gametime::TimeMessage;
use crate::messaging::{InputMessage, StateMessage, InitialInformation, ServerInputMessage};

#[derive(Serialize, Deserialize, Debug)]
pub enum ToClientMessageUDP<StateType, InputType, ServerInputType>
    where InputType: Clone,
          StateType: Clone,
          ServerInputType: Clone {

    //TODO: see if these can be borrowed
    TimeMessage(TimeMessage),
    InputMessage(InputMessage<InputType>),
    ServerInputMessage(ServerInputMessage<ServerInputType>),
    StateMessage(StateMessage<StateType>)
}