use crate::interface::{
    GameFactoryTrait,
    GameTrait,
};
use commons::factory::FactoryTrait;
use commons::threading::eventhandling;

pub type Factory<GameFactory> = <GameFactory as GameFactoryTrait>::Factory;
pub type TcpReader<GameFactory> = <Factory<GameFactory> as FactoryTrait>::TcpReader;
pub type UdpSocket<GameFactory> = <Factory<GameFactory> as FactoryTrait>::UdpSocket;

//TODO: remove this
pub type EventSender<T> = eventhandling::EventHandlerSender<T>;

pub type Receiver<GameFactory, T> = <Factory<GameFactory> as FactoryTrait>::Receiver<T>;

pub type Game<GameFactory> = <GameFactory as GameFactoryTrait>::Game;
pub type InterpolationResult<GameFactory> = <Game<GameFactory> as GameTrait>::InterpolationResult;
pub type ClientInputEvent<GameFactory> = <Game<GameFactory> as GameTrait>::ClientInputEvent;
