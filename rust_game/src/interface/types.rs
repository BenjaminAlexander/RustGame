use commons::factory::FactoryTrait;
use crate::interface::GameFactoryTrait;

pub type TcpSender<GameFactory> = <<GameFactory as GameFactoryTrait>::Factory as FactoryTrait>::TcpSender;
pub type TcpReceiver<GameFactory> = <<GameFactory as GameFactoryTrait>::Factory as FactoryTrait>::TcpReceiver;