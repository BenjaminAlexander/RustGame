use crate::gamemanager::{Data, ManagerObserverTrait, StepMessage};
use crate::interface::GameTrait;
use crate::messaging::{ServerInputMessage, StateMessage};
use crate::server::ServerCore;
use crate::server::udpoutput::UdpOutput;
use crate::threading::{ChannelDrivenThreadSender as Sender, Consumer};

pub struct ServerManagerObserver<Game: GameTrait> {
    server_core_sender: Sender<ServerCore<Game>>,
    udp_outputs: Vec<Sender<UdpOutput<Game>>>,
    render_receiver_sender: Sender<Data<Game>>
}

impl<Game: GameTrait> ServerManagerObserver<Game> {

    pub fn new(server_core_sender: Sender<ServerCore<Game>>,
           udp_outputs: Vec<Sender<UdpOutput<Game>>>,
           render_receiver_sender: Sender<Data<Game>>) -> Self {

        Self {
            server_core_sender,
            udp_outputs,
            render_receiver_sender
        }

    }

}

impl<Game: GameTrait> ManagerObserverTrait for ServerManagerObserver<Game> {
    type Game = Game;

    const IS_SERVER: bool = true;

    fn on_step_message(&self, step_message: StepMessage<Game>) {
        self.render_receiver_sender.on_step_message(step_message);
    }

    fn on_completed_step(&self, state_message: StateMessage<Game>) {

        for udp_output in self.udp_outputs.iter() {
            udp_output.on_completed_step(state_message.clone());
        }
    }

    fn on_server_input_message(&self, server_input_message: ServerInputMessage<Game>) {

        for udp_output in self.udp_outputs.iter() {
            udp_output.accept(server_input_message.clone());
        }
    }
}