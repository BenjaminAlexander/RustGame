use commons::factory::FactoryTrait;
use commons::threading::eventhandling;
use crate::interface::GameFactoryTrait;

pub type Factory<GameFactory> = <GameFactory as GameFactoryTrait>::Factory;
pub type TcpWriter<GameFactory> = <Factory<GameFactory> as FactoryTrait>::TcpWriter;
pub type TcpReader<GameFactory> = <Factory<GameFactory> as FactoryTrait>::TcpReader;
pub type UdpSocket<GameFactory> = <Factory<GameFactory> as FactoryTrait>::UdpSocket;
pub type EventSender<GameFactory, T> = eventhandling::EventSender<Factory<GameFactory>, T>;