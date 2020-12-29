use crate::messaging::InputMessage;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ToServerMessage<InputType>
    where InputType: Clone {

    Input(InputMessage<InputType>)
}