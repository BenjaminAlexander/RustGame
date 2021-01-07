use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait State<InputType: Input>: Serialize + DeserializeOwned + Clone + Debug + Send + 'static {

    fn get_next_state(&self, inputs: &Vec<Option<InputType>>) -> Self;

}

pub trait Input: Serialize + DeserializeOwned + Clone + Debug + Send + 'static {

}