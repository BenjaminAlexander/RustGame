use serde::{
    de::DeserializeOwned,
    Serialize,
};
use std::fmt::Debug;

pub trait AggregateInput: Send {
    type InputEvent: Send + 'static;

    type InputAggregate: Serialize + DeserializeOwned + Clone + Debug + Send + 'static;

    fn new() -> Self;

    //TODO: make type for fractional frame duration
    fn aggregate_input_event(&mut self, fractional_frame_duration: f64, input_event: Self::InputEvent) /* -> Option<Self::InputEvent> */;

    //TODO: remove
    fn peak_input(&self) -> Self::InputAggregate;

    //TODO: rename
    fn reset_for_new_frame(&mut self) /* -> Self::InputEvent */;
}
