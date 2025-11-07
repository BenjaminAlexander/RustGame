use crate::interface::{
    GameFactoryTrait,
    GameTrait,
};
use commons::threading::eventhandling;

pub type Factory<GameFactory> = <GameFactory as GameFactoryTrait>::Factory;

//TODO: remove this
pub type EventSender<T> = eventhandling::EventHandlerSender<T>;

pub type Game<GameFactory> = <GameFactory as GameFactoryTrait>::Game;
pub type InterpolationResult<GameFactory> = <Game<GameFactory> as GameTrait>::InterpolationResult;
pub type ClientInputEvent<GameFactory> = <Game<GameFactory> as GameTrait>::ClientInputEvent;
