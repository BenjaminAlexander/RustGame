use crate::gamemanager::CoreSenderTrait;
use crate::gametime::TimeMessage;
use crate::interface::GameTrait;
use crate::messaging::StateMessage;
use crate::server::ServerCore;
use crate::server::udpoutput::UdpOutput;
use crate::threading::Sender;

pub struct ServerManagerObserver<Game: GameTrait> {
    server_core_sender: Sender<ServerCore<Game>>,
    udp_outputs: Vec<Sender<UdpOutput<Game>>>
}

impl<Game: GameTrait> CoreSenderTrait for ServerManagerObserver<Game> {
    type Game = Game;

    fn on_time_message(&self, time_message: TimeMessage) {
        self.server_core_sender.on_time_message(time_message);
    }

    fn on_completed_step(&self, state_message: StateMessage<Game>) {
        self.server_core_sender.on_completed_step(state_message);
    }
}