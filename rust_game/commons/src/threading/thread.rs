pub trait Thread: Sized + Send + 'static {
    fn run(self);
}
