use crate::frame_manager::{ClientProvidedInput, FrameIndexAndState, InputCoordinates, ObserveFrames, PresentInput};
use crate::messaging::ToClientInputMessage;
use crate::server::udpoutput::UdpOutput;
use crate::state_channel::StateSender;
use crate::GameTrait;
use log::warn;
use std::ops::ControlFlow;

pub struct ServerManagerObserver<Game: GameTrait> {
    udp_outputs: Vec<UdpOutput<Game>>,
    state_sender: StateSender<Game>,
}

impl<Game: GameTrait> ServerManagerObserver<Game> {
    pub fn new(udp_outputs: Vec<UdpOutput<Game>>, state_sender: StateSender<Game>) -> Self {
        return Self {
            udp_outputs,
            state_sender,
        };
    }
}

impl<Game: GameTrait> ObserveFrames for ServerManagerObserver<Game> {
    type Game = Game;

    const IS_SERVER: bool = true;

    fn new_state(
        &self,
        is_state_authoritative: bool,
        state_message: FrameIndexAndState<Game>,
    ) -> ControlFlow<()> {
        let result = self.state_sender.send_state(state_message.clone());

        if result.is_err() {
            warn!("Failed to send StepMessage to Render Receiver");
            return ControlFlow::Break(());
        }

        if is_state_authoritative {
            for udp_output in self.udp_outputs.iter() {
                let result = udp_output.send_completed_step(state_message.clone());

                if result.is_err() {
                    warn!("Failed to send CompletedStep to UdpOutput");
                    return ControlFlow::Break(());
                }
            }
        }

        ControlFlow::Continue(())
    }

    fn input_authoritatively_missing(
        &self,
        input_coordinates: InputCoordinates,
    ) -> ControlFlow<()> {
        for udp_output in self.udp_outputs.iter() {
            let result = udp_output.send_input_message(ToClientInputMessage::new(
                input_coordinates,
                PresentInput::AuthoritativeMissing,
            ));

            if result.is_err() {
                warn!("Failed to send CompletedStep to UdpOutput");
                return ControlFlow::Break(());
            }
        }

        ControlFlow::Continue(())
    }
    
    fn input_authoritative(
        &self,
        input_coordinates: InputCoordinates,
        input: <Self::Game as GameTrait>::InputAggregate
    ) -> ControlFlow<()> {
        for udp_output in self.udp_outputs.iter() {
            let result = udp_output.send_input_message(ToClientInputMessage::new(
                input_coordinates,
                PresentInput::ClientProvided(ClientProvidedInput::Authoritative(input.clone())),
            ));

            if result.is_err() {
                warn!("Failed to send CompletedStep to UdpOutput");
                return ControlFlow::Break(());
            }
        }

        ControlFlow::Continue(())
    }
}
