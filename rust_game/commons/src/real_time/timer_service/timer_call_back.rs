use std::ops::DerefMut;

pub trait TimerCallBack: Send + 'static {
    //TODO: tick should be passed something that can
    // * Reschedule the timer
    // * Get the current time
    // * Get the time this was scheduled for
    fn tick(&mut self);
}

impl<T: Fn() + Send + 'static> TimerCallBack for T {
    fn tick(&mut self) {
        (self)();
    }
}

impl TimerCallBack for Box<dyn TimerCallBack> {
    fn tick(&mut self) {
        self.deref_mut().tick();
    }
}
