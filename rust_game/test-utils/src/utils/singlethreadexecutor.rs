use commons::{
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
use std::sync::{
    Arc,
    Condvar,
    Mutex,
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
                let (mutex, condvar) = join_signal_clone.as_ref();
                *mutex.lock().unwrap() = false;
                condvar.notify_all();
            })
            .unwrap();

        return Self {
            join_signal,
            sender,
        };
    }

    pub fn execute_runnable(self, runnable: Runnable) -> Result<(), Runnable> {
        return match self.sender.send_event(runnable) {
            Ok(()) => Ok(()),
            Err(EventOrStopThread::Event(runnable)) => Err(runnable),
            Err(EventOrStopThread::StopThread) => panic!("Illegal State"),
        };
    }

    pub fn execute_function<T: FnOnce() + Send + 'static>(
        self,
        function: T,
    ) -> Result<(), Runnable> {
        return self.execute_runnable(Box::new(function));
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
            ChannelEvent::Timeout => EventHandleResult::WaitForNextEvent(self),
            ChannelEvent::ChannelEmpty => EventHandleResult::WaitForNextEvent(self),
            ChannelEvent::ChannelDisconnected => EventHandleResult::StopThread(()),
        }
    }

    fn on_stop(self, _: ReceiveMetaData) -> Self::ThreadReturn {
        return ();
    }
}
