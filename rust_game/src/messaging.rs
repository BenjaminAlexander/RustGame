use serde::{Deserialize, Serialize};

pub use self::inputmessage::InputMessage;
pub use self::toservermessagetcp::ToServerMessageTCP;
pub use self::toservermessageudp::ToServerMessageUDP;
pub use self::toclientmessagetcp::ToClientMessageTCP;
pub use self::toclientmessageudp::ToClientMessageUDP;
pub use self::statemessage::StateMessage;
pub use self::initialinformation::InitialInformation;
pub use self::messagefragment::MessageFragment;

mod toservermessagetcp;
mod inputmessage;
mod toclientmessagetcp;
mod statemessage;
mod initialinformation;
mod toclientmessageudp;
mod toservermessageudp;
mod messagefragment;

pub const MAX_UDP_DATAGRAM_SIZE: usize = 1000;
