use crate::gamemanager::ManagerObserverTrait;
use crate::interface::RenderReceiverMessage;
use crate::messaging::{StateMessage, ToClientInputMessage};
use crate::server::udpoutput::UdpOutput;
use crate::{FrameIndex, GameTrait};
use commons::real_time::Sender;

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

impl<Game: GameTrait> ManagerObserverTrait for ServerManagerObserver<Game> {
    type Game = Game;

    const IS_SERVER: bool = true;

    fn on_step_message(&self, is_state_authoritative: bool, state_message: StateMessage<Game>) {

        let send_result = self
            .render_receiver_sender
            .send(RenderReceiverMessage::StepMessage(state_message.clone()));

        //TODO: handle without panic
        if send_result.is_err() {
            panic!("Failed to send StepMessage to Render Receiver");
        }

        if is_state_authoritative {
            for udp_output in self.udp_outputs.iter() {
                let result = udp_output.send_completed_step(state_message.clone());

                //TODO: handle without panic
                if result.is_err() {
                    panic!("Failed to send CompletedStep to UdpOutput");
                }
            }
        }
    }
    
    fn on_input_authoritatively_missing(&self, frame_index: FrameIndex, player_index: usize) {
        for udp_output in self.udp_outputs.iter() {
            let result = udp_output.send_input_message(ToClientInputMessage::new(frame_index, player_index, None));

            //TODO: handle without panic
            if result.is_err() {
                panic!("Failed to send CompletedStep to UdpOutput");
            }
        }
    }
}
