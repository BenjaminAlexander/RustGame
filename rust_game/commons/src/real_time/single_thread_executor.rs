use crate::real_time::{
    EventHandleResult,
    EventHandlerBuilder,
    EventSender,
    Factory,
    HandleEvent,
    ReceiveMetaData,
};
use std::sync::{
    Arc,
    Condvar,
    Mutex,
};

type Runnable = Box<dyn FnOnce() + Send>;

#[derive(Clone)]
pub struct SingleThreadExecutor {
    join_signal: Arc<(Mutex<bool>, Condvar)>,
    sender: EventSender<Runnable>,
}

impl SingleThreadExecutor {
    pub fn new() -> Self {
        let join_signal = Arc::new((Mutex::new(true), Condvar::new()));
        let join_signal_clone = join_signal.clone();

        let factory = Factory::new();

        let sender = EventHandlerBuilder::new(&factory)
            .spawn_thread_with_callback(
                "SingleThreadExecutor".to_string(),
                SingleThreadExecutorEventHandler(),
                move |_| {
                    let (wait_for_join_mutex, condvar) = join_signal_clone.as_ref();
                    *wait_for_join_mutex.lock().unwrap() = false;
                    condvar.notify_all();
                },
            )
            .unwrap();

        return Self {
            join_signal,
            sender,
        };
    }

    pub fn execute_runnable(&self, runnable: Runnable) -> Result<(), Runnable> {
        return self.sender.send_event(runnable);
    }

    pub fn execute_function<T: FnOnce() + Send + 'static>(
        &self,
        function: T,
    ) -> Result<(), Runnable> {
        return self.execute_runnable(Box::new(function));
    }

    pub fn execute_function_or_panic<T: FnOnce() + Send + 'static>(&self, function: T) {
        Self::panic_on_err(self.execute_function(function));
    }

    fn panic_on_err(result: Result<(), Runnable>) {
        if result.is_err() {
            panic!("Failed to send function to the executor");
        }
    }

    pub fn stop(&self) {
        let _ = self.sender.send_stop_thread();
    }

    pub fn wait_for_join(&self) {
        let (wait_for_join_mutex, condvar) = &*self.join_signal;
        let mut wait_for_join = wait_for_join_mutex.lock().unwrap();
        while *wait_for_join {
            wait_for_join = condvar.wait(wait_for_join).unwrap();
        }
    }
}

struct SingleThreadExecutorEventHandler();

impl HandleEvent for SingleThreadExecutorEventHandler {
    type Event = Runnable;
    type ThreadReturn = ();

    fn on_event(&mut self, _: ReceiveMetaData, event: Self::Event) -> EventHandleResult {
        event();
        return EventHandleResult::WaitForNextEvent;
    }
    
    fn on_stop_self(self) -> Self::ThreadReturn {
        return ();
    }
}

#[test]
#[should_panic]
fn test_panic_on_err() {
    SingleThreadExecutor::panic_on_err(Result::Err(Box::new(|| {})));
}
