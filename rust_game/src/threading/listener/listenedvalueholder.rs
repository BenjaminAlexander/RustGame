use crate::threading::listener::ListenerTrait;

pub struct ListenedValueHolder<T: ListenerTrait> {
    pub(super) value: T::ListenFor
}

impl<T: ListenerTrait> ListenedValueHolder<T> {

    pub fn get_value(&self) -> &T::ListenFor {
        return &self.value;
    }

    pub fn move_value(self) -> T::ListenFor {
        return self.value;
    }
}