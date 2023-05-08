use std::net::Ipv4Addr;
use std::str::FromStr;
use commons::factory::FactoryTrait;
use commons::threading::AsyncJoin;
use commons::threading::channel::SendError;
use commons::threading::eventhandling::{EventOrStopThread, EventSenderTrait};
use crate::client::{ClientCore, ClientCoreEvent};
use crate::client::ClientCoreEvent::{Connect, OnInputEvent};
use crate::gamemanager::RenderReceiver;
use crate::interface::{ClientInputEvent, EventSender, Factory, GameFactoryTrait};

pub struct Client<GameFactory: GameFactoryTrait> {
    core_sender: EventSender<GameFactory, ClientCoreEvent<GameFactory>>,
}

impl<GameFactory: GameFactoryTrait> Client<GameFactory> {

    pub fn new(factory: Factory<GameFactory>) -> (Self, RenderReceiver<GameFactory>) {

        let client_core_thread_builder = factory.new_thread_builder()
            .name("ClientCore")
            .build_channel_for_event_handler::<ClientCore<GameFactory>>();

        let (render_receiver_sender, render_receiver) = RenderReceiver::<GameFactory>::new(factory.clone());

        client_core_thread_builder.get_sender().send_event(Connect(render_receiver_sender.clone())).unwrap();

        let core_sender = client_core_thread_builder.get_sender().clone();

        client_core_thread_builder.spawn_event_handler(
            ClientCore::<GameFactory>::new(
                factory,
                Ipv4Addr::from_str("127.0.0.1").unwrap(),
                core_sender.clone(),
                render_receiver_sender
            ),
            AsyncJoin::log_async_join
        ).unwrap();

        let client = Self {
            core_sender
        };

        return (client, render_receiver);
    }

    pub fn send_client_input_event(&self, client_input_event: ClientInputEvent<GameFactory>) -> Result<(), ClientInputEvent<GameFactory>> {
        return match self.core_sender.send_event(OnInputEvent(client_input_event)) {
            Ok(()) => Ok(()),
            Err(e) => {
                let (_, event_or_stop) = e.0;

                match event_or_stop {
                    EventOrStopThread::Event(OnInputEvent(client_input_event)) => Err(client_input_event),
                    _ => panic!("This should never happen.")
                }
            }
        };
    }
}