use crate::messaging::InputMessage;
use std::time::SystemTime;
use crate::interface::Input;

#[derive(Debug)]
pub struct TimedInputMessage<InputType>
    where InputType: Input {

    inputMessage: InputMessage<InputType>,
    timeReceived: SystemTime
}

impl<InputType> TimedInputMessage<InputType>
    where InputType: Input {

    pub fn new(inputMessage: InputMessage<InputType>, timeReceived: SystemTime) -> Self {
        TimedInputMessage{inputMessage, timeReceived}
    }
}

impl<InputType> Clone for TimedInputMessage<InputType>
    where InputType: Input {

    fn clone(&self) -> Self {
        Self {
            inputMessage: self.inputMessage.clone(),
            timeReceived: self.timeReceived.clone()
        }
    }
}