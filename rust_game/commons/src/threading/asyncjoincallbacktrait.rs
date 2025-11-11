use crate::threading::AsyncJoin;

pub trait AsyncJoinCallBackTrait<T>: Send + 'static {
    fn join(self, async_join: AsyncJoin<T>);
}

impl<T, U: FnOnce(AsyncJoin<T>) + Send + 'static> AsyncJoinCallBackTrait<T> for U {
    fn join(self, async_join: AsyncJoin<T>) {
        (self)(async_join);
    }
}
