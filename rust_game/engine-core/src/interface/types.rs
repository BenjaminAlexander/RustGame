use crate::interface::{
    GameFactoryTrait,
    GameTrait,
};

pub type Factory<GameFactory> = <GameFactory as GameFactoryTrait>::Factory;
pub type Game<GameFactory> = <GameFactory as GameFactoryTrait>::Game;
pub type InterpolationResult<GameFactory> = <Game<GameFactory> as GameTrait>::InterpolationResult;
pub type ClientInputEvent<GameFactory> = <Game<GameFactory> as GameTrait>::ClientInputEvent;
