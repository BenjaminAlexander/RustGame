use crate::Play;
use crate::client::ClientCore;
use crate::messaging::ToClientMessageTCP;
use crate::state_channel::StateSender;
use commons::real_time::{EventHandlerStopper, Factory};
use commons::real_time::net::tcp::{HandleTcpRead, TcpReadHandlerBuilder, TcpReader};
use log::{
    info,
    warn,
};
use std::io::Error;
use std::ops::ControlFlow;
use std::ops::ControlFlow::*;

pub struct TcpInput {
    stopper: EventHandlerStopper
}

impl TcpInput {

    pub fn new<Game: Play>(
        factory: &Factory,
        tcp_reader: TcpReader,
        client_core: ClientCore<Game>,
        state_sender: StateSender<Game>,
    ) -> Result<Self, Error> {
        let tcp_read_handler = TcpReadHandler {
            player_index: None,
            client_core,
            state_sender,
        };

        let stopper = TcpReadHandlerBuilder::new_thread(
            &factory,
            "ClientTcpInput".to_string(),
            tcp_reader,
            tcp_read_handler,
        )?;

        Ok(Self{stopper})
    }

    pub fn stop(&self) -> Result<(), ()> {
        self.stopper.send_stop_thread()
    }
}

struct TcpReadHandler<Game: Play> {
    player_index: Option<usize>,
    client_core: ClientCore<Game>,
    state_sender: StateSender<Game>,
}

impl<Game: Play> HandleTcpRead for TcpReadHandler<Game> {
    type ReadType = ToClientMessageTCP<Game>;

    fn on_read(&mut self, message: Self::ReadType) -> ControlFlow<()> {
        match message {
            ToClientMessageTCP::InitialInformation(initial_information_message) => {
                info!(
                    "InitialInformation Received.  Player Index: {:?}",
                    initial_information_message.get_player_index()
                );

                self.player_index = Some(initial_information_message.get_player_index());

                let send_result = self.client_core.set_initial_information(initial_information_message.clone());

                if send_result.is_err() {
                    warn!("Failed to send InitialInformation to Core");
                    return Break(());
                }

                //TODO: move this into client core
                let send_result = self
                    .state_sender
                    .send_initial_information(initial_information_message);

                if send_result.is_err() {
                    warn!("Failed to send InitialInformation to Render Receiver");
                    return Break(());
                }
            }
        }

        return Continue(());
    }
}
