use serde::{Deserialize, Serialize};

use crate::gametime::TimeMessage;
use crate::messaging::{InputMessage, StateMessage};

#[derive(Serialize, Deserialize, Debug)]
pub enum ToClientMessage<StateType, InputType>
    where InputType: Clone,
          StateType: Clone {

    //TODO: see if these can be borrowed
    TimeMessage(TimeMessage),
    InputMessage(InputMessage<InputType>),
    StateMessage(StateMessage<StateType, InputType>)
}