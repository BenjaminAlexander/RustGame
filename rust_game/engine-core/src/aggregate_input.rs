use std::fmt::Debug;
use serde::{Serialize, de::DeserializeOwned};

pub trait AggregateInput: Send {

    type ClientInputEvent: Send + 'static;

    type ClientInput: Serialize + DeserializeOwned + Clone + Debug + Send + 'static;

    fn new() -> Self;

    fn handle_input_event(&mut self, input_event: Self::ClientInputEvent);

    fn get_input(&mut self) -> Self::ClientInput;
}