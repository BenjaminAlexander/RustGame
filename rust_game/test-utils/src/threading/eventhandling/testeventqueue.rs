use std::ops::ControlFlow;
use std::ops::ControlFlow::{Break, Continue};
use commons::threading::channel::{ReceiveMetaData, SendMetaData};
use commons::threading::eventhandling::{ChannelEvent, ChannelEventResult, EventHandlerTrait, WaitOrTryForNextEvent};
use commons::time::TimeValue;
/*
enum EventOrTimeout<T> {
    //TODO: use timestap from send metadata instead of tuple in queue
    Event(SendMetaData, T),
    Timeout
}

struct TestEventQueue<T: EventHandlerTrait> {
    event_handler: T,
    timeout: Option<TimeValue>,

    //TODO: make this a vec deque
    queue: Vec<(TimeValue, SendMetaData, T::Event)>
}

impl<T: EventHandlerTrait> TestEventQueue<T> {

    fn insert(&mut self, time_value: TimeValue, send_meta_data: SendMetaData, event :T::Event) {

        if let Some(timeout) = self.timeout.as_ref() {
            if !timeout.is_before(&time_value) {
                self.timeout = None;
            }
        }

        /* TODO: this could be improved by using binary search.  The default vec binary_search return
            a deterministic but unpredictable index when there are multiple elements as the same index.
            This implementation depends on getting the last index when there are multiple  */
        let mut insert_index = 0;
        for i in 0..self.queue.len() {
            if let Some((i_time_value, _, _)) = self.queue.get(i) {
                if time_value.is_before(i_time_value) {
                    insert_index = i;
                    break;
                }
            }
        }

        self.queue.insert(insert_index, (time_value, send_meta_data, event));
    }

    fn get_time_of_next_event(&self) -> Option<&TimeValue> {

        if let Some(timeout) = self.timeout.as_ref() {
            return Some(timeout);
        } else {
            return match self.queue.get(0) {
                Some((time_value, _, _)) => Some(time_value),
                None => self.timeout.as_ref()
            }
        }
    }

    fn handle_next_event(&mut self) {
        if !self.queue.is_empty() {
            let (time_value, send_meta_data, event) = self.queue.remove(0);
            //TODO: this event should happen at the right (Simulated) time

            let event_result = match event_or_timeout {
                EventOrTimeout::Event(send_meta_data, event) => self.event_handler.on_channel_event(ChannelEvent::ReceivedEvent(ReceiveMetaData::new(send_meta_data), event)),
                EventOrTimeout::Timeout => self.event_handler.on_channel_event(ChannelEvent::Timeout)
            };



            self.handle_event_result(event_result);
        }
    }

    fn handle_event_result(&mut self, event_result: ChannelEventResult<T>) -> ControlFlow<T::ThreadReturn, T> {
        return match event_result? {
            WaitOrTryForNextEvent::TryForNextEvent(event_handler) => self.handle_event_result(event_handler.on_channel_event(ChannelEvent::ChannelEmpty)),
            WaitOrTryForNextEvent::WaitForNextEvent(event_handler) => Continue(event_handler),
            WaitOrTryForNextEvent::WaitForNextEventOrTimeout(event_handler, timeout_duration) => self.insert(TimeValue::now().add(timeout_duration), event_handler)
        }

    }
}
*/
