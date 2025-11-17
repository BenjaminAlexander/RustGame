use crate::client::{
    ClientCore,
    ClientCoreEvent,
};
use crate::interface::{
    ClientInputEvent,
    Factory,
    GameFactoryTrait,
    RenderReceiver,
};
use commons::real_time::{
    EventHandlerBuilder,
    EventSender, FactoryTrait,
};
use std::net::Ipv4Addr;
use std::str::FromStr;

pub struct Client<GameFactory: GameFactoryTrait> {
    core_sender: EventSender<ClientCoreEvent<GameFactory>>,
}

impl<GameFactory: GameFactoryTrait> Client<GameFactory> {
    pub fn new(factory: Factory<GameFactory>) -> (Self, RenderReceiver<GameFactory>) {
        let client_core_thread_builder =
            EventHandlerBuilder::<ClientCore<GameFactory>>::new(&factory);

        let (render_receiver_sender, render_receiver) =
            RenderReceiver::<GameFactory>::new(&factory);

        let core_sender = client_core_thread_builder.get_sender().clone();

        client_core_thread_builder
            .spawn_thread(
                "ClientCore".to_string(),
                ClientCore::<GameFactory>::new(
                    factory.clone(),
                    Ipv4Addr::from_str("127.0.0.1").unwrap(),
                    core_sender.clone(),
                    render_receiver_sender,
                ),
            )
            .unwrap();

        let client = Self { core_sender };

        return (client, render_receiver);
    }

    pub fn send_client_input_event(
        &self,
        client_input_event: ClientInputEvent<GameFactory>,
    ) -> Result<(), ClientInputEvent<GameFactory>> {
        return match self
            .core_sender
            .send_event(ClientCoreEvent::OnInputEvent(client_input_event))
        {
            Ok(()) => Ok(()),
            Err(ClientCoreEvent::OnInputEvent(client_input_event)) => Err(client_input_event),
            _ => panic!("This should never happen."),
        };
    }
}
