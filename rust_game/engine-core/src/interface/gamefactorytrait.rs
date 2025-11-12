use crate::interface::GameTrait;
use commons::real_time::FactoryTrait;

//TODO: remove
pub trait GameFactoryTrait: Send + 'static {
    type Game: GameTrait;
    type Factory: FactoryTrait;
}
