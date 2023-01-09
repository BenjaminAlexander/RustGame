use crate::client::ClientCore;
use crate::gamemanager::RenderReceiverMessage;
use crate::gametime::{GameTimerObserverTrait, TimeMessage};
use crate::interface::GameTrait;
use crate::threading::{ChannelDrivenThreadSender, ValueSender};

pub struct ClientGameTimerObserver<Game: GameTrait> {
    core_sender: ChannelDrivenThreadSender<ClientCore<Game>>,
    render_receiver_sender: ValueSender<RenderReceiverMessage<Game>>
}

impl<Game: GameTrait> ClientGameTimerObserver<Game> {

    pub fn new(core_sender: ChannelDrivenThreadSender<ClientCore<Game>>,
               render_receiver_sender: ValueSender<RenderReceiverMessage<Game>>) -> Self {

        Self {
            core_sender,
            render_receiver_sender
        }

    }
}

impl<Game: GameTrait> GameTimerObserverTrait for ClientGameTimerObserver<Game> {
    type Game = Game;

    fn on_time_message(&self, time_message: TimeMessage) {
        self.core_sender.on_time_message(time_message.clone());
        self.render_receiver_sender.send(RenderReceiverMessage::TimeMessage(time_message.clone())).unwrap();
    }
}
