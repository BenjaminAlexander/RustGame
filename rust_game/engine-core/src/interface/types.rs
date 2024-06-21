use crate::interface::{GameFactoryTrait, GameTrait};
use commons::factory::FactoryTrait;
use commons::threading::eventhandling;

pub type Factory<GameFactory> = <GameFactory as GameFactoryTrait>::Factory;
pub type TcpWriter<GameFactory> = <Factory<GameFactory> as FactoryTrait>::TcpWriter;
pub type TcpReader<GameFactory> = <Factory<GameFactory> as FactoryTrait>::TcpReader;
pub type UdpSocket<GameFactory> = <Factory<GameFactory> as FactoryTrait>::UdpSocket;
pub type EventSender<GameFactory, T> = eventhandling::EventHandlerSender<Factory<GameFactory>, T>;
pub type Receiver<GameFactory, T> = <Factory<GameFactory> as FactoryTrait>::Receiver<T>;
pub type Sender<GameFactory, T> = <Factory<GameFactory> as FactoryTrait>::Sender<T>;

pub type Game<GameFactory> = <GameFactory as GameFactoryTrait>::Game;
pub type InterpolationResult<GameFactory> = <Game<GameFactory> as GameTrait>::InterpolationResult;
pub type ClientInputEvent<GameFactory> = <Game<GameFactory> as GameTrait>::ClientInputEvent;
