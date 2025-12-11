use crate::client::{
    ClientCore,
    ClientCoreEvent,
};
use crate::{GameTrait, StateReceiver};
use commons::real_time::{
    EventHandlerBuilder,
    EventSender,
    Factory,
};
use std::net::Ipv4Addr;
use std::str::FromStr;

pub struct Client<Game: GameTrait> {
    core_sender: EventSender<ClientCoreEvent<Game>>,
}

impl<Game: GameTrait> Client<Game> {
    pub fn new(factory: Factory) -> (Self, StateReceiver<Game>) {
        let client_core_thread_builder = EventHandlerBuilder::<ClientCore<Game>>::new(&factory);

        let (render_receiver_sender, render_receiver) = StateReceiver::<Game>::new(&factory);

        let core_sender = client_core_thread_builder.get_sender().clone();

        client_core_thread_builder
            .spawn_thread(
                "ClientCore".to_string(),
                ClientCore::<Game>::new(
                    factory,
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
        client_input_event: Game::ClientInputEvent,
    ) -> Result<(), Game::ClientInputEvent> {
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
