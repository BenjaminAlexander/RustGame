use crate::gamemanager::Data;
use crate::gametime::{GameTimerObserverTrait, TimeMessage};
use crate::interface::GameTrait;
use crate::server::ServerCore;
use crate::threading::{Consumer, Sender};

pub struct ServerGameTimerObserver<Game: GameTrait> {
    core_sender: Sender<ServerCore<Game>>,
    render_receiver_sender: Sender<Data<Game>>
}

impl<Game: GameTrait> ServerGameTimerObserver<Game> {

    pub fn new(core_sender: Sender<ServerCore<Game>>,
               render_receiver_sender: Sender<Data<Game>>) -> Self {

        Self {
            core_sender,
            render_receiver_sender
        }

    }
}

impl<Game: GameTrait> GameTimerObserverTrait for ServerGameTimerObserver<Game> {
    type Game = Game;

    fn on_time_message(&self, time_message: TimeMessage) {
        self.core_sender.on_time_message(time_message.clone());
        self.render_receiver_sender.accept(time_message.clone());
    }
}