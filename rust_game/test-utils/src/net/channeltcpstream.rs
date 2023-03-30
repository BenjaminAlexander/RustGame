use std::any::Any;
use commons::factory::FactoryTrait;
use commons::threading::channel::{Receiver, SenderTrait};

pub struct ChannelTcpStream<Factory: FactoryTrait> {
    sender: Factory::Sender<Box<dyn Any + Send>>,
    receiver: Receiver<Factory, Box<dyn Any + Send>>
}

impl<Factory: FactoryTrait> ChannelTcpStream<Factory> {
    pub fn new(factory: Factory) -> (Self, Self) {
        let (sender_1, receiver_1) = factory.new_channel::<Box<dyn Any + Send>>().take();
        let (sender_2, receiver_2) = factory.new_channel::<Box<dyn Any + Send>>().take();

        let stream_1 = Self {
            sender: sender_1,
            receiver: receiver_2
        };

        let stream_2 = Self {
            sender: sender_2,
            receiver: receiver_1
        };

        return (stream_1, stream_2);
    }
}