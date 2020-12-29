use serde::{Serialize, Deserialize};
use super::InputMessage;

#[derive(Serialize, Deserialize, Debug)]
pub enum ToServerMessage<InputType>
    where InputType: Clone {

    Input(InputMessage<InputType>)
}