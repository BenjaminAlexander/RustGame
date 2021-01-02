use serde::{Deserialize, Serialize};

use crate::gametime::TimeMessage;

#[derive(Serialize, Deserialize, Debug)]
pub enum ToClientMessage {

    TimeMessage(TimeMessage)
}