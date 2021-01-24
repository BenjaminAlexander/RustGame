use serde::{Deserialize, Serialize};

pub use self::inputmessage::InputMessage;
pub use self::toservermessage::ToServerMessage;
pub use self::toclientmessage::ToClientMessage;
pub use self::statemessage::StateMessage;
pub use self::initialinformation::InitialInformation;

mod toservermessage;
mod inputmessage;
mod toclientmessage;
mod statemessage;
mod initialinformation;

