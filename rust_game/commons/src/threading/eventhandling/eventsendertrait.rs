use crate::factory::FactoryTrait;
use crate::threading::eventhandling::SendResult;

pub trait EventSenderTrait<T> {

    fn send_event(&self, factory: &impl FactoryTrait, event: T) -> SendResult<T>;

    fn send_stop_thread(&self, factory: &impl FactoryTrait) -> SendResult<T>;
}