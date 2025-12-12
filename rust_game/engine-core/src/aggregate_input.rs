use std::fmt::Debug;
use serde::{Serialize, de::DeserializeOwned};

pub trait AggregateInput: Send {

    type ClientInputEvent: Send + 'static;

    type ClientInput: Serialize + DeserializeOwned + Clone + Debug + Send + 'static;

    fn new() -> Self;

    fn aggregate_input_event(&mut self, input_event: Self::ClientInputEvent);

    fn peak_input(&self) -> Self::ClientInput;

    fn reset_for_new_frame(&mut self);
}