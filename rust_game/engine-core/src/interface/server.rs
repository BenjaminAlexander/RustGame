use std::io::Error;

use crate::{
    Play,
    StateReceiver,
    server::ServerCore,
};
use commons::real_time::Factory;

/// A sender to a locally running server
#[derive(Clone)]
pub struct Server<Game: Play> {
    server_core: ServerCore<Game>,
}

impl<Game: Play> Server<Game> {
    /// Starts a new locally running server.  The server immediately starts listening
    /// for and accepting connections from clients.
    pub fn new(factory: Factory) -> Result<(Self, StateReceiver<Game>), Error> {
        let (state_sender, state_receiver) = StateReceiver::new(&factory);

        let server_core = ServerCore::new(factory.clone(), state_sender)?;

        let server = Self { server_core };

        return Ok((server, state_receiver));
    }

    /// Starts the game.  This will stop the server from accepting new connections
    /// from clients.
    pub fn start_game(&self) -> Result<(), ()> {
        self.server_core.start_game()
    }

    /// Stops the server
    pub fn stop(&self) -> Result<(), ()> {
        self.server_core.stop()
    }
}
