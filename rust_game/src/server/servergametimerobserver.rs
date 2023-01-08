use crate::gamemanager::RenderReceiverMessage;
use crate::gametime::{GameTimerObserverTrait, TimeMessage};
use crate::interface::GameTrait;
use crate::server::ServerCore;
use crate::server::udpoutput::UdpOutput;
use crate::threading::{ChannelDrivenThreadSender as Sender, ValueSender};

pub struct ServerGameTimerObserver<Game: GameTrait> {
    core_sender: Sender<ServerCore<Game>>,
    render_receiver_sender: ValueSender<RenderReceiverMessage<Game>>,
    udp_outputs: Vec<Sender<UdpOutput<Game>>>,
}

impl<Game: GameTrait> ServerGameTimerObserver<Game> {

    pub fn new(core_sender: Sender<ServerCore<Game>>,
               render_receiver_sender: ValueSender<RenderReceiverMessage<Game>>,
               udp_outputs: Vec<Sender<UdpOutput<Game>>>) -> Self {

        Self {
            core_sender,
            render_receiver_sender,
            udp_outputs
        }

    }
}

impl<Game: GameTrait> GameTimerObserverTrait for ServerGameTimerObserver<Game> {
    type Game = Game;

    fn on_time_message(&self, time_message: TimeMessage) {

        for udp_output in self.udp_outputs.iter() {
            udp_output.on_time_message(time_message.clone());
        }

        self.core_sender.on_time_message(time_message.clone());
        self.render_receiver_sender.send(RenderReceiverMessage::TimeMessage(time_message.clone())).unwrap();
    }
}
