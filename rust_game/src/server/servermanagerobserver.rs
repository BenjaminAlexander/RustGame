use crate::gamemanager::{ManagerObserverTrait, RenderReceiverMessage, StepMessage};
use crate::interface::GameTrait;
use crate::messaging::{ServerInputMessage, StateMessage};
use crate::server::udpoutput::UdpOutputEvent;
use crate::threading::{channel, eventhandling};

pub struct ServerManagerObserver<Game: GameTrait> {
    udp_outputs: Vec<eventhandling::Sender<UdpOutputEvent<Game>>>,
    render_receiver_sender: channel::Sender<RenderReceiverMessage<Game>>
}

impl<Game: GameTrait> ServerManagerObserver<Game> {

    pub fn new(udp_outputs: Vec<eventhandling::Sender<UdpOutputEvent<Game>>>,
               render_receiver_sender: channel::Sender<RenderReceiverMessage<Game>>) -> Self {

        Self {
            udp_outputs,
            render_receiver_sender
        }

    }

}

impl<Game: GameTrait> ManagerObserverTrait for ServerManagerObserver<Game> {
    type Game = Game;

    const IS_SERVER: bool = true;

    fn on_step_message(&self, step_message: StepMessage<Game>) {
        self.render_receiver_sender.send(RenderReceiverMessage::StepMessage(step_message)).unwrap();
    }

    fn on_completed_step(&self, state_message: StateMessage<Game>) {

        for udp_output in self.udp_outputs.iter() {
            udp_output.send_event(UdpOutputEvent::SendCompletedStep(state_message.clone())).unwrap();
        }
    }

    fn on_server_input_message(&self, server_input_message: ServerInputMessage<Game>) {

        for udp_output in self.udp_outputs.iter() {
            udp_output.send_event(UdpOutputEvent::SendServerInputMessage(server_input_message.clone())).unwrap();
        }
    }
}
