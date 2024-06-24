pub trait EventSenderTrait<T> {
    fn send_event(&self, event: T) -> Result<(), T>;

    fn send_stop_thread(&self) -> Result<(), ()>;
}
