use serde::{Deserialize, Serialize};

use super::InputMessage;

#[derive(Serialize, Deserialize, Debug)]
pub enum ToServerMessage<InputType>
    where InputType: Clone {

    //TODO: see if these can be borrowed
    Input(InputMessage<InputType>)
}