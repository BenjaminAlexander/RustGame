use commons::factory::FactoryTrait;
use crate::interface::GameTrait;

pub trait GameFactoryTrait: 'static {
    type Game: GameTrait;
    type Factory: FactoryTrait;
}