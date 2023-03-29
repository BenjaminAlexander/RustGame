use commons::factory::FactoryTrait;
use crate::interface::GameFactoryTrait;

pub type TcpStream<GameFactory> = <<GameFactory as GameFactoryTrait>::Factory as FactoryTrait>::TcpStream;