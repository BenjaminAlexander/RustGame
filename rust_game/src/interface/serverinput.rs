use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

pub trait ServerInput: Serialize + DeserializeOwned + Clone + Debug + Send + 'static {

}