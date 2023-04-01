use commons::factory::FactoryTrait;
use crate::interface::GameFactoryTrait;

pub type TcpWriter<GameFactory> = <<GameFactory as GameFactoryTrait>::Factory as FactoryTrait>::TcpWriter;
pub type TcpReader<GameFactory> = <<GameFactory as GameFactoryTrait>::Factory as FactoryTrait>::TcpReader;