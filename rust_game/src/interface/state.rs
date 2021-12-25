use crate::interface::{Input, ClientUpdateArg};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

pub trait State: Serialize + DeserializeOwned + Clone + Debug + Send + Sync + 'static {

    fn new(player_count: usize) -> Self;

}