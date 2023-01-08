use crate::threading::eventhandling::SentEventHolder;

pub struct ReceivedEventHolder<T> {
    pub(crate) sent_event_holder: SentEventHolder<T>
}

impl<T> ReceivedEventHolder<T> {

    pub fn get_event(&self) -> &T { &self.sent_event_holder.event }

    pub fn move_event(self) -> T { self.sent_event_holder.event }
}