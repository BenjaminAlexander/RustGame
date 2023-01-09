use crate::gametime::TimeValue;
use crate::threading::listener::ListenerTrait;

pub struct ListenedValueHolder<T: ListenerTrait> {
    value: T::ListenFor,
    time_received: TimeValue
}

impl<T: ListenerTrait> ListenedValueHolder<T> {

    pub fn new(value: T::ListenFor) -> Self {
        ListenedValueHolder {
            value,
            time_received: TimeValue::now()
        }
    }

    pub fn get_time_received(&self) -> TimeValue { self.time_received }

    pub fn get_value(&self) -> &T::ListenFor { &self.value }

    pub fn move_value(self) -> T::ListenFor { self.value }
}