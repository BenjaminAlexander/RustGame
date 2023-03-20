pub trait TimerCallBack: Send + 'static {
    fn tick(&mut self);
}

impl<T: Fn() + Send + 'static> TimerCallBack for T {
    fn tick(&mut self) {
        (self)();
    }
}

impl TimerCallBack for Box<dyn TimerCallBack> {
    fn tick(&mut self) {
        (*self).tick();
    }
}