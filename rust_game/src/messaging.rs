use serde::{Deserialize, Serialize};

pub use self::inputmessage::InputMessage;
pub use self::toservermessagetcp::ToServerMessageTCP;
pub use self::toservermessageudp::ToServerMessageUDP;
pub use self::toclientmessagetcp::ToClientMessageTCP;
pub use self::toclientmessageudp::ToClientMessageUDP;
pub use self::statemessage::StateMessage;
pub use self::initialinformation::InitialInformation;

mod toservermessagetcp;
mod inputmessage;
mod toclientmessagetcp;
mod statemessage;
mod initialinformation;
mod toclientmessageudp;
mod toservermessageudp;

pub const MAX_UDP_DATAGRAM_SIZE: usize = 100;
