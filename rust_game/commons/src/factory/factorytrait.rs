use std::sync::mpsc;
use crate::threading::channel::{SenderTrait, SendMetaData};
use crate::time::TimeValue;

pub trait FactoryTrait: Clone + Send + 'static {
    type Sender<T: Send>: SenderTrait<T>;

    fn now(&self) -> TimeValue;

    fn new_sender<T: Send>(&self, sender: mpsc::Sender<(SendMetaData, T)>) -> Self::Sender<T>;
}