mod nextstatearg;

pub use self::nextstatearg::NextStateArg;

use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait State<InputType: Input<InputEventType>, InputEventType: InputEvent>: Serialize + DeserializeOwned + Clone + Debug + Send + 'static {

    fn new(player_count: usize) -> Self;

    fn get_next_state(&self, arg: &NextStateArg<InputType, InputEventType>) -> Self;

}

pub trait Input<InputEventType: InputEvent>: Serialize + DeserializeOwned + Clone + Debug + Send + 'static {

    fn new() -> Self;

    fn accumulate(&mut self, input_event: InputEventType);

}

pub trait InputEvent: Send + 'static {

}