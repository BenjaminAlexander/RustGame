mod sender;
mod receiver;
mod channel;

pub(crate) use self::sender::{Sender, SentValueHolder, SendError};
pub(crate) use self::receiver::{Receiver, TryRecvError, RecvError, ReceivedValueHolder};
pub(crate) use self::channel::message_channel;