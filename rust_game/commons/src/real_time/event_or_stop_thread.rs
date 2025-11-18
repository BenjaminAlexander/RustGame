pub enum EventOrStopThread<T> {
    Event(T),
    StopThread,
}
