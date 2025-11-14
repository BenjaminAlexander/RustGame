//TODO: hide EventOrStopThread enum
pub enum EventOrStopThread<T> {
    Event(T),
    StopThread,
}
