pub trait TimerCallBack: Send + 'static {
    fn tick(&mut self);
}