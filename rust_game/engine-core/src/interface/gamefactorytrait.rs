use crate::interface::GameTrait;
use commons::factory::FactoryTrait;

pub trait GameFactoryTrait: Send + 'static {
    type Game: GameTrait;
    type Factory: FactoryTrait;
}
