use crate::interface::{Input, NextStateArg};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

pub trait State: Serialize + DeserializeOwned + Clone + Debug + Send + 'static {

    fn new(player_count: usize) -> Self;

}