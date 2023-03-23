use std::marker::PhantomData;
use commons::factory::RealFactory;
use crate::interface::{GameFactoryTrait, GameTrait};

#[derive(Clone, Copy)]
pub struct RealGameFactory<Game: GameTrait> {
    phantom: PhantomData<Game>
}

impl<Game: GameTrait> GameFactoryTrait for RealGameFactory<Game> {
    type Game = Game;
    type Factory = RealFactory;
}