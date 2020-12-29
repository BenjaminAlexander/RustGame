use std::{thread, mem};
use std::thread::{JoinHandle, Builder};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use log::{info, warn};

pub trait MessageHandler<MessageType> : Send {
    fn handle(&self, message: MessageType);
}

pub struct RunnableThread<MessageType: Send>
    where MessageType: Send {

    joinHandle: JoinHandle<()>,
    sender: Sender<MessageType>
}

impl<MessageType: 'static> RunnableThread<MessageType>
    where MessageType: Send {
    pub fn new<HandlerType: 'static + MessageHandler<MessageType>>(name: String, messageHandler: HandlerType) -> RunnableThread<MessageType> {

        let (sender, rx): (Sender<MessageType>, Receiver<MessageType>) = mpsc::channel();
        let method = move || {
            info!("Starting");

            for message in rx.iter() {
                messageHandler.handle(message);
            }

            info!("Ending");
        };

        let builder:Builder = Builder::new().name(name);

        let joinHandle = builder.spawn(method).unwrap();

        RunnableThread{joinHandle, sender}
    }

    pub fn send(&self, function:MessageType) {
        self.sender.send(function);
    }
}
