use crate::messaging::InputMessage;
use std::time::SystemTime;

#[derive(Debug)]
pub struct TimedInputMessage<InputType>
    where InputType: Clone {

    inputMessage: InputMessage<InputType>,
    timeReceived: SystemTime
}

impl<InputType> TimedInputMessage<InputType>
    where InputType: Clone {

    pub fn new(inputMessage: InputMessage<InputType>, timeReceived: SystemTime) -> Self {
        TimedInputMessage{inputMessage, timeReceived}
    }
}

impl<InputType> Clone for TimedInputMessage<InputType>
    where InputType: Clone {

    fn clone(&self) -> Self {
        Self {
            inputMessage: self.inputMessage.clone(),
            timeReceived: self.timeReceived.clone()
        }
    }
}