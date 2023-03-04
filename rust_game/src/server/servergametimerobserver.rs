use crate::gamemanager::RenderReceiverMessage;
use crate::gametime::{GameTimerObserverTrait, TimeMessage};
use crate::interface::GameTrait;
use crate::server::servercore::ServerCoreEvent;
use crate::server::udpoutput::UdpOutputEvent;
use commons::threading::{channel, eventhandling};

pub struct ServerGameTimerObserver<Game: GameTrait> {
    core_sender: eventhandling::Sender<ServerCoreEvent<Game>>,
    render_receiver_sender: channel::Sender<RenderReceiverMessage<Game>>,
    udp_outputs: Vec<eventhandling::Sender<UdpOutputEvent<Game>>>,
}

impl<Game: GameTrait> ServerGameTimerObserver<Game> {

    pub fn new(core_sender: eventhandling::Sender<ServerCoreEvent<Game>>,
               render_receiver_sender: channel::Sender<RenderReceiverMessage<Game>>,
               udp_outputs: Vec<eventhandling::Sender<UdpOutputEvent<Game>>>) -> Self {

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
            udp_output.send_event(UdpOutputEvent::SendTimeMessage(time_message.clone())).unwrap();
        }

        self.core_sender.send_event(ServerCoreEvent::TimeMessageEvent(time_message.clone())).unwrap();
        self.render_receiver_sender.send(RenderReceiverMessage::TimeMessage(time_message.clone())).unwrap();
    }
}
