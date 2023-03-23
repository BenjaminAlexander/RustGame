use crate::time::TimeValue;

pub trait FactoryTrait: Clone + Send + 'static {
    fn now(&self) -> TimeValue;
}