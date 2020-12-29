use std::time::SystemTime;

use crate::interface::Input;
use crate::messaging::InputMessage;

#[derive(Debug)]
pub struct TimedInputMessage<InputType>
    where InputType: Input {

    input_message: InputMessage<InputType>,
    time_received: SystemTime
}

impl<InputType> TimedInputMessage<InputType>
    where InputType: Input {

    pub fn new(input_message: InputMessage<InputType>, time_received: SystemTime) -> Self {
        TimedInputMessage{ input_message, time_received }
    }
}

impl<InputType> Clone for TimedInputMessage<InputType>
    where InputType: Input {

    fn clone(&self) -> Self {
        Self {
            input_message: self.input_message.clone(),
            time_received: self.time_received.clone()
        }
    }
}