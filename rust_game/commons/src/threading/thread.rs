pub trait Thread: Sized + Send + 'static {
    type ReturnType: Send + 'static;

    fn run(self) -> Self::ReturnType;
}
