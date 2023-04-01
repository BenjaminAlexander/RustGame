use commons::factory::FactoryTrait;
use crate::interface::GameFactoryTrait;

//TODO: rename as reader
pub type TcpSender<GameFactory> = <<GameFactory as GameFactoryTrait>::Factory as FactoryTrait>::TcpSender;

//TODO: rename as writer
pub type TcpReceiver<GameFactory> = <<GameFactory as GameFactoryTrait>::Factory as FactoryTrait>::TcpReceiver;