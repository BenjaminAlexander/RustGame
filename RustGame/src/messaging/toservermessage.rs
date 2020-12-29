use serde::{Deserialize, Serialize};

use super::InputMessage;

#[derive(Serialize, Deserialize, Debug)]
pub enum ToServerMessage<InputType>
    where InputType: Clone {

    Input(InputMessage<InputType>)
}