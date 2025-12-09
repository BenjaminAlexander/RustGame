use crate::frame_manager::ObserveFrames;
use crate::interface::RenderReceiverMessage;
use crate::messaging::{
    FrameIndexAndState,
    ToClientInputMessage,
};
use crate::server::udpoutput::UdpOutput;
use crate::{
    FrameIndex,
    GameTrait,
};
use commons::real_time::Sender;
use log::warn;
use std::ops::ControlFlow;

pub struct ServerManagerObserver<Game: GameTrait> {
    udp_outputs: Vec<UdpOutput<Game>>,
    render_receiver_sender: Sender<RenderReceiverMessage<Game>>,
}

impl<Game: GameTrait> ServerManagerObserver<Game> {
    pub fn new(
        udp_outputs: Vec<UdpOutput<Game>>,
        render_receiver_sender: Sender<RenderReceiverMessage<Game>>,
    ) -> Self {
        return Self {
            udp_outputs,
            render_receiver_sender,
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
        let result = self
            .render_receiver_sender
            .send(RenderReceiverMessage::StepMessage(state_message.clone()));

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
        frame_index: FrameIndex,
        player_index: usize,
    ) -> ControlFlow<()> {
        for udp_output in self.udp_outputs.iter() {
            let result = udp_output.send_input_message(ToClientInputMessage::new(
                frame_index,
                player_index,
                None,
            ));

            if result.is_err() {
                warn!("Failed to send CompletedStep to UdpOutput");
                return ControlFlow::Break(());
            }
        }

        ControlFlow::Continue(())
    }
}
