use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait State<InputType: Input<InputEventType>, InputEventType: InputEvent>: Serialize + DeserializeOwned + Clone + Debug + Send + 'static {

    fn get_next_state(&self, inputs: &Vec<Option<InputType>>) -> Self;

}

pub trait Input<InputEventType: InputEvent>: Serialize + DeserializeOwned + Clone + Debug + Send + 'static {

    fn new() -> Self;

    fn accumulate(&mut self, input_event: InputEventType);

}

pub trait InputEvent: Send + 'static {

}