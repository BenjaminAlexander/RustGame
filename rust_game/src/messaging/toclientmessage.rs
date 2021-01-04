use serde::{Deserialize, Serialize};

use crate::gametime::TimeMessage;
use crate::messaging::InputMessage;

#[derive(Serialize, Deserialize, Debug)]
pub enum ToClientMessage<InputType>
    where InputType: Clone {

    TimeMessage(TimeMessage),
    InputMessage(InputMessage<InputType>)
}