use crate::client::ClientCoreEvent;
use crate::client::ClientCoreEvent::OnTimeMessage;
use crate::gamemanager::RenderReceiverMessage;
use crate::gametime::{GameTimerObserverTrait, TimeMessage};
use crate::interface::GameTrait;
use commons::threading::channel::Sender;
use commons::threading::eventhandling;

pub struct ClientGameTimerObserver<Game: GameTrait> {
    core_sender: eventhandling::Sender<ClientCoreEvent<Game>>,
    render_receiver_sender: Sender<RenderReceiverMessage<Game>>
}

impl<Game: GameTrait> ClientGameTimerObserver<Game> {

    pub fn new(core_sender: eventhandling::Sender<ClientCoreEvent<Game>>,
               render_receiver_sender: Sender<RenderReceiverMessage<Game>>) -> Self {

        Self {
            core_sender,
            render_receiver_sender
        }

    }
}

impl<Game: GameTrait> GameTimerObserverTrait for ClientGameTimerObserver<Game> {
    type Game = Game;

    fn on_time_message(&self, time_message: TimeMessage) {
        self.core_sender.send_event(OnTimeMessage(time_message.clone())).unwrap();
        self.render_receiver_sender.send(RenderReceiverMessage::TimeMessage(time_message.clone())).unwrap();
    }
}
