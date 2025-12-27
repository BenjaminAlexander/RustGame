use crate::client::ClientCore;
use crate::{
    Play,
    StateReceiver,
};
use commons::real_time::Factory;
use std::io::Error;

/// A sender to a locally running client
pub struct Client<Game: Play> {
    client_core: ClientCore<Game>,
}

impl<Game: Play> Client<Game> {
    /// Starts a new locally running client.  The client will immediately attempt
    /// to connect to the server.
    pub fn new(factory: Factory) -> Result<(Self, StateReceiver<Game>), Error> {
        let (state_sender, state_receiver) = StateReceiver::<Game>::new(&factory);

        let client_core = ClientCore::new(factory, state_sender)?;

        let client = Self { client_core };

        Ok((client, state_receiver))
    }

    /// Sends a new [InputEvent][Play::InputEvent] to the local client
    pub fn send_input_event(&self, input_event: Game::InputEvent) -> Result<(), ()> {
        self.client_core.input_event(input_event)
    }
}
