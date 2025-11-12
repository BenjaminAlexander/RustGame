use crate::interface::{
    GameFactoryTrait,
    GameTrait,
};
use commons::real_time::RealFactory;
use std::marker::PhantomData;

#[derive(Clone, Copy)]
pub struct RealGameFactory<Game: GameTrait> {
    phantom: PhantomData<Game>,
}

impl<Game: GameTrait> GameFactoryTrait for RealGameFactory<Game> {
    type Game = Game;
    type Factory = RealFactory;
}
