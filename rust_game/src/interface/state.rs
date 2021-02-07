use crate::interface::{Input, NextStateArg};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

pub trait State<InputType: Input>: Serialize + DeserializeOwned + Clone + Debug + Send + 'static {

    fn new(player_count: usize) -> Self;

    fn get_next_state(&self, arg: &NextStateArg<InputType>) -> Self;

}