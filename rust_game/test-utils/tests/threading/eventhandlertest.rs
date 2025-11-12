use commons::{
    logging::setup_test_logging, real_time::{EventHandlerBuilder, RealFactory}, threading::{
        channel::ReceiveMetaData,
        eventhandling::{
            ChannelEvent,
            EventHandleResult,
            EventHandlerTrait,
        },
    }, time::TimeDuration
};
use test_utils::{
    assert::AsyncExpects,
};

const NUMBER: i32 = 87;

struct Count {
    count: i32,
}

#[derive(Debug)]
enum CountEvent {
    Add(i32),
    Subtract(i32),
    WaitForTimeout,
}

impl EventHandlerTrait for Count {
    type Event = CountEvent;

    type ThreadReturn = i32;

    fn on_channel_event(
        &mut self,
        channel_event: ChannelEvent<CountEvent>,
    ) -> EventHandleResult<Self> {
        match channel_event {
            ChannelEvent::ReceivedEvent(_, CountEvent::Add(x)) => self.count += x,
            ChannelEvent::ReceivedEvent(_, CountEvent::Subtract(x)) => self.count -= x,
            ChannelEvent::ReceivedEvent(_, CountEvent::WaitForTimeout) => {
                return EventHandleResult::WaitForNextEventOrTimeout(TimeDuration::from_millis_f64(
                    100.0,
                ))
            }
            ChannelEvent::Timeout => return EventHandleResult::StopThread(NUMBER),
            ChannelEvent::ChannelDisconnected => return EventHandleResult::StopThread(self.count),
            _ => { /*no-op*/ }
        }

        return EventHandleResult::WaitForNextEvent;
    }

    fn on_stop(self, _receive_meta_data: ReceiveMetaData) -> i32 {
        return self.count;
    }
}

#[test]
fn test_async_join() {
    setup_test_logging();

    let event_handler = Count { count: 0 };

    let async_expects = AsyncExpects::new();
    let expect_five = async_expects.new_async_expect("Five", 5);
    let join_call_back = move |result: i32| {
        expect_five.set_actual(result);
    };

    let real_factory = RealFactory::new();

    let sender = EventHandlerBuilder::new(&real_factory)
        .spawn_thread_with_callback("EventHandler".to_string(), event_handler, join_call_back)
        .unwrap();

    sender.send_event(CountEvent::Add(7)).unwrap();
    sender.send_event(CountEvent::Subtract(2)).unwrap();
    sender.send_stop_thread().unwrap();

    async_expects.wait_for_all();
}

#[test]
fn test_no_timeout() {
    setup_test_logging();

    let event_handler = Count { count: 0 };

    let async_expects = AsyncExpects::new();
    let expect_five = async_expects.new_async_expect("Five", 5);
    let join_call_back = move |result: i32| {
        expect_five.set_actual(result);
    };

    let real_factory = RealFactory::new();

    let sender = EventHandlerBuilder::new(&real_factory)
        .spawn_thread_with_callback("EventHandler".to_string(), event_handler, join_call_back)
        .unwrap();

    sender.send_event(CountEvent::Add(7)).unwrap();
    sender.send_event(CountEvent::WaitForTimeout).unwrap();
    sender.send_event(CountEvent::Subtract(2)).unwrap();
    sender.send_stop_thread().unwrap();

    async_expects.wait_for_all();
}

#[test]
fn test_timeout() {
    setup_test_logging();

    let event_handler = Count { count: 0 };

    let async_expects = AsyncExpects::new();
    let expect_five = async_expects.new_async_expect("A Number", NUMBER);
    let join_call_back = move |result: i32| {
        expect_five.set_actual(result);
    };

    let real_factory = RealFactory::new();

    let sender = EventHandlerBuilder::new(&real_factory)
        .spawn_thread_with_callback("EventHandler".to_string(), event_handler, join_call_back)
        .unwrap();

    sender.send_event(CountEvent::Add(7)).unwrap();
    sender.send_event(CountEvent::Subtract(2)).unwrap();
    sender.send_event(CountEvent::WaitForTimeout).unwrap();

    async_expects.wait_for_all();
}

#[test]
fn test_drop_sender_while_waiting_for_timeout() {
    setup_test_logging();

    let event_handler = Count { count: 0 };

    let async_expects = AsyncExpects::new();
    let expect_five = async_expects.new_async_expect("Five", 5);
    let join_call_back = move |result: i32| {
        expect_five.set_actual(result);
    };

    {
        let real_factory = RealFactory::new();

        let sender = EventHandlerBuilder::new(&real_factory)
            .spawn_thread_with_callback("EventHandler".to_string(), event_handler, join_call_back)
            .unwrap();

        sender.send_event(CountEvent::Add(7)).unwrap();
        sender.send_event(CountEvent::Subtract(2)).unwrap();
        sender.send_event(CountEvent::WaitForTimeout).unwrap();

        //Drop the sender
    }

    async_expects.wait_for_all();
}
