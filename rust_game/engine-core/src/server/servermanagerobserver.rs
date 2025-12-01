use crate::gamemanager::{ManagerObserverTrait, StateMessageType};
use crate::interface::RenderReceiverMessage;
use crate::messaging::StateMessage;
use crate::server::udpoutput::UdpOutput;
use crate::GameTrait;
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

    fn on_step_message(&self, message_type: StateMessageType, state_message: StateMessage<Game>) {

        let send_result = self
            .render_receiver_sender
            .send(RenderReceiverMessage::StepMessage(state_message.clone()));

        //TODO: handle without panic
        if send_result.is_err() {
            panic!("Failed to send StepMessage to Render Receiver");
        }

        if message_type.is_authoritative() {
            for udp_output in self.udp_outputs.iter() {
                let send_result = udp_output.send_completed_step(state_message.clone());

                //TODO: handle without panic
                if send_result.is_err() {
                    panic!("Failed to send CompletedStep to UdpOutput");
                }
            }
        }
    }
}
