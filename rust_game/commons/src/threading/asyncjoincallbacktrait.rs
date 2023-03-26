use crate::factory::FactoryTrait;
use crate::threading::AsyncJoin;

pub trait AsyncJoinCallBackTrait<Factory: FactoryTrait, T>: Send + 'static {

    fn join(self, asyncJoin: AsyncJoin<Factory, T>);

}

impl<Factory: FactoryTrait, T, U: FnOnce(AsyncJoin<Factory, T>) + Send + 'static> AsyncJoinCallBackTrait<Factory, T> for U {
    fn join(self, asyncJoin: AsyncJoin<Factory, T>) {
        (self)(asyncJoin);
    }
}