use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::Serialize;

pub trait State: Serialize + DeserializeOwned + Clone + Debug + Send + 'static {

}

pub trait Input: Serialize + DeserializeOwned + Clone + Debug + Send + 'static {

}