use crate::factory::FactoryTrait;
use crate::threading::channel::{
    SendMetaData,
    SenderTrait,
};
use std::sync::mpsc::{
    self,
    SendError,
};

pub struct RealSender<Factory: FactoryTrait, T: Send> {
    factory: Factory,
    sender: mpsc::Sender<(SendMetaData, T)>,
}

impl<Factory: FactoryTrait, T: Send> RealSender<Factory, T> {
    pub fn new(factory: Factory, sender: mpsc::Sender<(SendMetaData, T)>) -> Self {
        return Self { factory, sender };
    }
}

impl<Factory: FactoryTrait, T: Send> SenderTrait<T> for RealSender<Factory, T> {
    fn send(&self, value: T) -> Result<(), T> {
        let send_meta_data = SendMetaData::new(&self.factory);

        return match self.sender.send((send_meta_data, value)) {
            Ok(()) => Result::Ok(()),
            Err(SendError((_, value))) => Result::Err(value),
        };
    }
}

impl<Factory: FactoryTrait, T: Send> Clone for RealSender<Factory, T> {
    fn clone(&self) -> Self {
        return Self {
            factory: self.factory.clone(),
            sender: self.sender.clone(),
        };
    }
}
