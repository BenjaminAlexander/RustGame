use serde::{Deserialize, Serialize};

use crate::gametime::TimeMessage;
use crate::messaging::{InputMessage, StateMessage, InitialInformation};

#[derive(Serialize, Deserialize, Debug)]
pub enum ToClientMessageUDP<StateType, InputType>
    where InputType: Clone,
          StateType: Clone {

    //TODO: see if these can be borrowed
    TimeMessage(TimeMessage),
    InputMessage(InputMessage<InputType>),
    StateMessage(StateMessage<StateType>)
}