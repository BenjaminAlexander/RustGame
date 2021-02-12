use serde::{Deserialize, Serialize};

use super::InputMessage;

#[derive(Serialize, Deserialize, Debug)]
pub enum ToServerMessageTCP {

    //TODO: see if these can be borrowed
    //InputX(InputMessage<InputType>)
}