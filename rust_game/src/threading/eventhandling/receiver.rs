use crate::threading::eventhandling::eventhandler::EventHandlerTrait;
use crate::threading::eventhandling::SentEventHolder;

pub struct ReceivedEventHolder<T: EventHandlerTrait> {
    pub(crate) sent_event_holder: SentEventHolder<T>
}

impl<T: EventHandlerTrait> ReceivedEventHolder<T> {

    pub fn get_event(&self) -> &T::Event { &self.sent_event_holder.event }

    pub fn move_event(self) -> T::Event { self.sent_event_holder.event }
}