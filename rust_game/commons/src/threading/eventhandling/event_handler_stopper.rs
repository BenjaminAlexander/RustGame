use crate::threading::eventhandling::EventSender;

pub struct EventHandlerStopper {
    sender: EventSender<()>,
}

impl EventHandlerStopper {
    pub(crate) fn new(sender: EventSender<()>) -> Self {
        return Self { sender };
    }

    pub fn send_stop_thread(&self) -> Result<(), ()> {
        return self.sender.send_stop_thread();
    }
}

impl Clone for EventHandlerStopper {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}
