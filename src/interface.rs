use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

pub trait State: Serialize + DeserializeOwned + Debug + Send + 'static {

}

pub trait Input: Serialize + DeserializeOwned + Clone + Debug + Send + 'static {

}