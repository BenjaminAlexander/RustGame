use crate::{
    factory::{
        FactoryTrait,
        RealFactory,
    },
    threading::{
        channel::{
            RealSender,
            ReceiveMetaData,
        },
        eventhandling::{
            ChannelEvent,
            EventHandleResult,
            EventHandlerTrait,
            EventOrStopThread,
            EventSenderTrait,
        },
    },
};
use std::{
    ops::Deref,
    sync::{
        Arc,
        Condvar,
        Mutex,
    },
};

type Runnable = Box<dyn FnOnce() + Send>;

#[derive(Clone)]
pub struct SingleThreadExecutor {
    join_signal: Arc<(Mutex<bool>, Condvar)>,
    sender: RealSender<RealFactory, EventOrStopThread<Runnable>>,
}

impl SingleThreadExecutor {
    pub fn new() -> Self {
        let join_signal = Arc::new((Mutex::new(true), Condvar::new()));
        let join_signal_clone = join_signal.clone();

        let factory = RealFactory::new();

        let sender = factory
            .new_thread_builder()
            .name("SingleThreadExecutor")
            .spawn_event_handler(SingleThreadExecutorEventHandler(), move |_| {
                let (wait_for_join_mutex, condvar) = join_signal_clone.as_ref();
                *wait_for_join_mutex.lock().unwrap() = false;
                condvar.notify_all();
            })
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

impl EventHandlerTrait for SingleThreadExecutorEventHandler {
    type Event = Runnable;
    type ThreadReturn = ();

    fn on_channel_event(self, channel_event: ChannelEvent<Self::Event>) -> EventHandleResult<Self> {
        match channel_event {
            ChannelEvent::ReceivedEvent(_, runnable) => {
                runnable();
                return EventHandleResult::WaitForNextEvent(self);
            }
            ChannelEvent::Timeout => unreachable!(),
            ChannelEvent::ChannelEmpty => EventHandleResult::WaitForNextEvent(self),
            ChannelEvent::ChannelDisconnected => EventHandleResult::StopThread(()),
        }
    }

    fn on_stop(self, _: ReceiveMetaData) -> Self::ThreadReturn {
        return ();
    }
}
